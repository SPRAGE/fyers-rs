//! Low-level demo: auth handshake + binary subscribe sent with the **raw**
//! `NSE:SBIN-EQ`-style symbol string (no HSM-token resolution). The server
//! acks the subscribe but does not emit market data because the topic name
//! is not a valid HSM token — useful for illustrating why
//! `data_socket_protocol_walkthrough.rs` performs the symbol-token lookup.
//! Saves raw frames to `/tmp` for analysis.

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
const CHANNEL_NUM: u8 = 11;

const REQ_AUTH: u8 = 0x01;
const REQ_SUBSCRIBE: u8 = 0x04;
#[allow(dead_code)]
const REQ_UNSUBSCRIBE: u8 = 0x05;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required");
    let access_token = std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required");
    let symbols: Vec<String> = std::env::var("FYERS_LIVE_SYMBOL")
        .unwrap_or_else(|_| "NSE:SBIN-EQ".to_owned())
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();

    let jwt = strip_appid_prefix(&access_token);
    let hsm_key = extract_hsm_key(jwt)?;
    println!("client_id={client_id} hsm_key.len={} symbols={symbols:?}", hsm_key.len());

    let request = DATA_SOCKET_URL.into_client_request()?;
    println!("connecting...");
    let (mut stream, _) = connect_async(request).await?;
    println!("upgraded.");

    let auth_msg = build_auth_message(&hsm_key, MODE_FULL, SOURCE_ID);
    println!("sending auth ({} bytes)...", auth_msg.len());
    stream.send(Message::Binary(Bytes::from(auth_msg))).await?;

    let auth_resp = read_next_binary(&mut stream, "auth response").await?;
    save_frame("/tmp/fyers_auth_resp.bin", &auth_resp);
    let parsed_auth = parse_envelope(&auth_resp)?;
    println!(
        "auth response: req_type=0x{:02x} fields={}",
        parsed_auth.req_type,
        parsed_auth.fields.len()
    );
    for (idx, f) in parsed_auth.fields.iter().enumerate() {
        let preview = preview_field(&f.value);
        println!("  field[{idx}] id={} len={} {preview}", f.id, f.value.len());
    }

    let sub_msg = build_subscribe_message(&symbols, CHANNEL_NUM, &access_token, SOURCE_ID);
    println!(
        "sending subscribe ({} bytes) for {symbols:?} on channel {CHANNEL_NUM}...",
        sub_msg.len()
    );
    save_frame("/tmp/fyers_sub_req.bin", &sub_msg);
    stream.send(Message::Binary(Bytes::from(sub_msg))).await?;

    println!("waiting for response frames (up to 5)...");
    for n in 1..=5 {
        match timeout(Duration::from_secs(8), stream.next()).await {
            Ok(Some(Ok(Message::Binary(b)))) => {
                let path = format!("/tmp/fyers_frame_{n}.bin");
                save_frame(&path, &b);
                println!("[#{n}] BINARY {} bytes -> {path}", b.len());
                match parse_envelope(&b) {
                    Ok(env) => {
                        println!(
                            "       envelope: req_type=0x{:02x} fields={}",
                            env.req_type,
                            env.fields.len()
                        );
                        for (idx, f) in env.fields.iter().enumerate() {
                            let preview = preview_field(&f.value);
                            println!("       field[{idx}] id={} len={} {preview}", f.id, f.value.len());
                        }
                    }
                    Err(e) => println!("       envelope parse failed: {e}"),
                }
            }
            Ok(Some(Ok(other))) => println!("[#{n}] non-binary: {other:?}"),
            Ok(Some(Err(e))) => {
                eprintln!("[#{n}] stream error: {e}");
                break;
            }
            Ok(None) => {
                println!("[#{n}] stream closed");
                break;
            }
            Err(_) => {
                println!("[#{n}] timeout (8s)");
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
    let payload_b64 = parts.next().ok_or_else(|| "JWT missing payload".to_string())?;
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
    let total = 18 + hsm_token.len() + source.len();
    let mut buf = Vec::with_capacity(total);
    let length_prefix = u16::try_from(total - 2).expect("auth msg under 64KiB");
    buf.extend_from_slice(&length_prefix.to_be_bytes());
    buf.push(REQ_AUTH);
    buf.push(0x04);
    push_field(&mut buf, 1, hsm_token.as_bytes());
    push_field(&mut buf, 2, &[mode]);
    push_field(&mut buf, 3, &[0x01]);
    push_field(&mut buf, 4, source.as_bytes());
    buf
}

fn build_subscribe_message(
    symbols: &[String],
    channel_num: u8,
    access_token: &str,
    source: &str,
) -> Vec<u8> {
    let mut scrips_data = Vec::new();
    let count = u16::try_from(symbols.len()).expect("under 64Ki symbols");
    scrips_data.extend_from_slice(&count.to_be_bytes());
    for s in symbols {
        let bytes = s.as_bytes();
        scrips_data.push(u8::try_from(bytes.len()).expect("symbol under 256 bytes"));
        scrips_data.extend_from_slice(bytes);
    }

    // Python: data_len = 18 + len(scrips_data) + len(access_token) + len(source).
    // Faithfully reproduce that arithmetic even though it overstates the true buffer length.
    let data_len_value = 18 + scrips_data.len() + access_token.len() + source.len();
    let data_len_u16 = u16::try_from(data_len_value).expect("subscribe header under 64KiB");

    let mut buf = Vec::new();
    buf.extend_from_slice(&data_len_u16.to_be_bytes());
    buf.push(REQ_SUBSCRIBE);
    buf.push(0x02);

    push_field(&mut buf, 1, &scrips_data);

    push_field(&mut buf, 2, &[channel_num]);

    buf
}

fn push_field(buf: &mut Vec<u8>, field_id: u8, value: &[u8]) {
    buf.push(field_id);
    let len = u16::try_from(value.len()).expect("field length under 64KiB");
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(value);
}

#[derive(Debug)]
struct Envelope {
    req_type: u8,
    fields: Vec<Field>,
}

#[derive(Debug)]
struct Field {
    id: u8,
    value: Vec<u8>,
}

fn parse_envelope(data: &[u8]) -> Result<Envelope, String> {
    if data.len() < 4 {
        return Err(format!("frame too short: {} bytes", data.len()));
    }
    let _data_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    let req_type = data[2];
    let field_count = data[3] as usize;
    let mut offset = 4;
    let mut fields = Vec::with_capacity(field_count);
    for i in 0..field_count {
        if offset + 3 > data.len() {
            return Err(format!("truncated at field {i}"));
        }
        let id = data[offset];
        let len = u16::from_be_bytes([data[offset + 1], data[offset + 2]]) as usize;
        offset += 3;
        if offset + len > data.len() {
            return Err(format!("field {i} length {len} exceeds frame"));
        }
        fields.push(Field {
            id,
            value: data[offset..offset + len].to_vec(),
        });
        offset += len;
    }
    Ok(Envelope { req_type, fields })
}

fn preview_field(value: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(value) {
        if s.chars().all(|c| !c.is_control() || c == '\n' || c == '\t') && !s.is_empty() {
            let truncated: String = s.chars().take(80).collect();
            return format!("text=\"{}{}\"", truncated, if s.len() > 80 { "\u{2026}" } else { "" });
        }
    }
    let hex: String = value
        .iter()
        .take(24)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("hex={hex}{}", if value.len() > 24 { " \u{2026}" } else { "" })
}

async fn read_next_binary<S>(stream: &mut S, label: &str) -> Result<Vec<u8>, String>
where
    S: futures_util::Stream<
            Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>,
        > + Unpin,
{
    match timeout(Duration::from_secs(10), stream.next()).await {
        Ok(Some(Ok(Message::Binary(b)))) => Ok(b.to_vec()),
        Ok(Some(Ok(other))) => Err(format!("{label}: non-binary frame {other:?}")),
        Ok(Some(Err(e))) => Err(format!("{label}: stream error {e}")),
        Ok(None) => Err(format!("{label}: stream closed")),
        Err(_) => Err(format!("{label}: timeout")),
    }
}

fn save_frame(path: &str, data: &[u8]) {
    if let Err(e) = std::fs::write(path, data) {
        eprintln!("warn: failed to save {path}: {e}");
    }
}
