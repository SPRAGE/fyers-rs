//! M1 proof-of-concept: connect to the Fyers V3 data socket, send the binary
//! auth handshake matching the official Python SDK byte-for-byte, and print
//! the first server response.
//!
//! This example bypasses `src/ws/data.rs` entirely because that module
//! implements an abstract JSON protocol that does not match the real wire
//! format. If this PoC succeeds, the real implementation is rebuilt around it.

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use futures_util::{SinkExt, StreamExt};
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Bytes, Message};

const DATA_SOCKET_URL: &str = "wss://socket.fyers.in/hsm/v1-5/prod";
const SOURCE_ID: &str = "fyers-rs/0.1.0";
const MODE_FULL: u8 = b'P';

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required");
    let access_token = std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required");

    let jwt = strip_appid_prefix(&access_token);
    let hsm_key = extract_hsm_key(jwt)?;
    println!(
        "client_id: {client_id}\nhsm_key (len {}): {}\u{2026}",
        hsm_key.len(),
        &hsm_key.chars().take(20).collect::<String>()
    );

    let auth_msg = build_auth_message(&hsm_key, MODE_FULL, SOURCE_ID);
    println!("auth message ({} bytes):\n{}", auth_msg.len(), hex_dump(&auth_msg));

    let request = DATA_SOCKET_URL.into_client_request()?;
    println!("connecting to {DATA_SOCKET_URL}...");
    let (mut stream, response) = connect_async(request).await?;
    println!(
        "WS upgrade ok: status {} headers {:?}",
        response.status(),
        response.headers().keys().collect::<Vec<_>>()
    );

    println!("sending binary auth frame...");
    stream
        .send(Message::Binary(Bytes::from(auth_msg)))
        .await?;

    println!("waiting for first server frame (10s)...");
    for n in 1..=5 {
        match timeout(Duration::from_secs(10), stream.next()).await {
            Ok(Some(Ok(msg))) => {
                describe_frame(n, &msg);
                if let Message::Close(_) = msg {
                    break;
                }
            }
            Ok(Some(Err(err))) => {
                eprintln!("[#{n}] stream error: {err}");
                break;
            }
            Ok(None) => {
                println!("[#{n}] stream closed");
                break;
            }
            Err(_) => {
                println!("[#{n}] timeout (no frame in 10s)");
                break;
            }
        }
    }

    let _ = stream.close(None).await;
    Ok(())
}

fn strip_appid_prefix(token: &str) -> &str {
    match token.find(':') {
        Some(idx) => &token[idx + 1..],
        None => token,
    }
}

fn extract_hsm_key(jwt: &str) -> Result<String, String> {
    let mut parts = jwt.split('.');
    let _header = parts.next().ok_or_else(|| "JWT missing header".to_string())?;
    let payload_b64 = parts
        .next()
        .ok_or_else(|| "JWT missing payload".to_string())?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|e| format!("base64 decode failed: {e}"))?;
    let payload: serde_json::Value =
        serde_json::from_slice(&payload_bytes).map_err(|e| format!("JSON decode failed: {e}"))?;
    payload
        .get("hsm_key")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| "JWT payload has no hsm_key field".to_string())
}

fn build_auth_message(hsm_token: &str, mode: u8, source: &str) -> Vec<u8> {
    let total_size = 18 + hsm_token.len() + source.len();
    let mut buf = Vec::with_capacity(total_size);

    let length_prefix = u16::try_from(total_size - 2).expect("auth message under 64KiB");
    buf.extend_from_slice(&length_prefix.to_be_bytes());
    buf.push(0x01);
    buf.push(0x04);

    push_field(&mut buf, 1, hsm_token.as_bytes());

    push_field(&mut buf, 2, &[mode]);

    push_field(&mut buf, 3, &[0x01]);

    push_field(&mut buf, 4, source.as_bytes());

    buf
}

fn push_field(buf: &mut Vec<u8>, field_id: u8, value: &[u8]) {
    buf.push(field_id);
    let len = u16::try_from(value.len()).expect("field length under 64KiB");
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(value);
}

fn describe_frame(seq: usize, msg: &Message) {
    match msg {
        Message::Text(t) => println!("[#{seq}] TEXT ({} bytes): {t}", t.len()),
        Message::Binary(b) => {
            println!("[#{seq}] BINARY ({} bytes):\n{}", b.len(), hex_dump(b));
            if let Some(parsed) = try_parse_auth_response(b) {
                println!("       parsed auth response: {parsed}");
            }
        }
        Message::Ping(b) => println!("[#{seq}] PING ({} bytes)", b.len()),
        Message::Pong(b) => println!("[#{seq}] PONG ({} bytes)", b.len()),
        Message::Close(frame) => println!("[#{seq}] CLOSE: {frame:?}"),
        Message::Frame(_) => println!("[#{seq}] RAW FRAME"),
    }
}

fn try_parse_auth_response(data: &[u8]) -> Option<String> {
    let mut offset = 4;
    if data.len() < offset + 3 {
        return None;
    }
    offset += 1;
    let len = u16::from_be_bytes([data.get(offset)?.clone(), data.get(offset + 1)?.clone()])
        as usize;
    offset += 2;
    let value = data.get(offset..offset + len)?;
    let s = std::str::from_utf8(value).ok()?;
    Some(format!("status={s} ({})", if s == "K" { "OK" } else { "NOT OK" }))
}

fn hex_dump(data: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let _ = write!(out, "  {:04x}  ", i * 16);
        for b in chunk {
            let _ = write!(out, "{b:02x} ");
        }
        for _ in chunk.len()..16 {
            out.push_str("   ");
        }
        out.push(' ');
        for b in chunk {
            let c = if (0x20..0x7f).contains(b) {
                *b as char
            } else {
                '.'
            };
            out.push(c);
        }
        out.push('\n');
    }
    out
}
