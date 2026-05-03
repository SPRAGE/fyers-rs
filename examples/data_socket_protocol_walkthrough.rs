//! Low-level walkthrough of the complete Fyers V3 data-socket protocol,
//! bypassing [`fyers_rs::FyersClient::data_socket`] to exercise each step
//! explicitly:
//!
//! 1. POST `https://api-t1.fyers.in/data/symbol-token` to convert
//!    `NSE:SBIN-EQ` → fytoken → `sf|nse_cm|<exch_token>` HSM symbol.
//! 2. Open the WebSocket and send the binary auth frame (`req_type=0x01`).
//! 3. Send the channel-mode set frame (`req_type=0x0c`) for full mode.
//! 4. Send the channel-resume frame (`req_type=0x08`).
//! 5. Send the binary subscribe frame (`req_type=0x04`) with HSM topics.
//! 6. Read responses, decode each envelope, save raw bytes to `/tmp`.
//!
//! Useful for protocol debugging and as documentation of the full
//! handshake. For ordinary streaming use the public client API — see
//! `examples/data_socket_symbol_update.rs`.
//!
//! Reference: official Python SDK (fyers-apiv3 3.1.12) `data_ws.py`.

use std::collections::HashMap;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use futures_util::{SinkExt, StreamExt};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Bytes, Message};

const DATA_SOCKET_URL: &str = "wss://socket.fyers.in/hsm/v1-5/prod";
const SYMBOL_TOKEN_API: &str = "https://api-t1.fyers.in/data/symbol-token";
const SOURCE_ID: &str = "fyers-rs/0.1.0";
const MODE_FULL: u8 = b'P';
const CHANNEL_NUM: u8 = 11;

const REQ_AUTH: u8 = 0x01;
const REQ_CHANNEL_RESUME: u8 = 0x08;
const REQ_FULL_MODE: u8 = 0x0c;
const REQ_SUBSCRIBE: u8 = 0x04;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = std::env::var("FYERS_CLIENT_ID").expect("FYERS_CLIENT_ID is required");
    let access_token_raw =
        std::env::var("FYERS_ACCESS_TOKEN").expect("FYERS_ACCESS_TOKEN is required");
    let symbols: Vec<String> = std::env::var("FYERS_LIVE_SYMBOL")
        .unwrap_or_else(|_| "NSE:SBIN-EQ".to_owned())
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();

    let jwt = strip_appid_prefix(&access_token_raw);
    let appid_token_header = format!("{client_id}:{jwt}");
    warn_if_token_kind_wrong(jwt);
    let hsm_key = extract_hsm_key(jwt)?;
    println!("client_id={client_id} hsm_key.len={} symbols={symbols:?}", hsm_key.len());

    println!("resolving symbols -> HSM tokens via {SYMBOL_TOKEN_API}...");
    let hsm_symbols = resolve_hsm_symbols(&symbols, &appid_token_header).await?;
    println!("resolved: {hsm_symbols:?}");
    if hsm_symbols.is_empty() {
        return Err("no symbols resolved".into());
    }

    let request = DATA_SOCKET_URL.into_client_request()?;
    println!("connecting to data socket...");
    let (mut stream, _) = connect_async(request).await?;

    let auth_msg = build_auth_message(&hsm_key, MODE_FULL, SOURCE_ID);
    println!("[1] sending auth ({} bytes)", auth_msg.len());
    stream.send(Message::Binary(Bytes::from(auth_msg))).await?;
    let auth_resp = read_next_binary(&mut stream, "auth resp").await?;
    save("/tmp/fyers_walkthrough_auth_resp.bin", &auth_resp);
    let env = parse_envelope(&auth_resp)?;
    print_envelope("[1] auth resp", &env);

    let full_mode = build_channel_bitmap_message(REQ_FULL_MODE, CHANNEL_NUM);
    println!("[2] sending full_mode ({} bytes)", full_mode.len());
    save("/tmp/fyers_walkthrough_full_mode_req.bin", &full_mode);
    stream.send(Message::Binary(Bytes::from(full_mode))).await?;

    let resume = build_channel_bitmap_message(REQ_CHANNEL_RESUME, CHANNEL_NUM);
    println!("[3] sending channel_resume ({} bytes)", resume.len());
    save("/tmp/fyers_walkthrough_resume_req.bin", &resume);
    stream.send(Message::Binary(Bytes::from(resume))).await?;

    let sub = build_subscribe_message(&hsm_symbols, CHANNEL_NUM, &access_token_raw, SOURCE_ID);
    println!("[4] sending subscribe ({} bytes) with HSM tokens", sub.len());
    save("/tmp/fyers_walkthrough_sub_req.bin", &sub);
    stream.send(Message::Binary(Bytes::from(sub))).await?;

    println!("waiting for response frames (up to 8)...");
    for n in 1..=8 {
        match timeout(Duration::from_secs(8), stream.next()).await {
            Ok(Some(Ok(Message::Binary(b)))) => {
                let path = format!("/tmp/fyers_walkthrough_frame_{n}.bin");
                save(&path, &b);
                println!("[#{n}] BINARY {} bytes -> {path}", b.len());
                match parse_envelope(&b) {
                    Ok(env) => print_envelope(&format!("[#{n}]"), &env),
                    Err(e) => println!("       envelope parse failed: {e}"),
                }
            }
            Ok(Some(Ok(Message::Ping(_)))) => println!("[#{n}] PING"),
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
                println!("[#{n}] timeout");
                break;
            }
        }
    }

    let _ = stream.close(None).await;
    Ok(())
}

async fn resolve_hsm_symbols(
    symbols: &[String],
    auth_header: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let exch_seg_dict: HashMap<&str, &str> = [
        ("1010", "nse_cm"),
        ("1011", "nse_fo"),
        ("1120", "mcx_fo"),
        ("1210", "bse_cm"),
        ("1012", "cde_fo"),
        ("1211", "bse_fo"),
        ("1212", "bcs_fo"),
        ("1020", "nse_com"),
    ]
    .into_iter()
    .collect();

    let client = reqwest::Client::new();
    let response = client
        .post(SYMBOL_TOKEN_API)
        .header(AUTHORIZATION, auth_header)
        .header(CONTENT_TYPE, "application/json")
        .json(&json!({"symbols": symbols}))
        .send()
        .await?;

    let body: serde_json::Value = response.json().await?;
    println!(
        "symbol-token API response: {}",
        serde_json::to_string_pretty(&body).unwrap_or_default()
    );

    let valid = body
        .get("validSymbol")
        .and_then(|v| v.as_object())
        .ok_or("validSymbol missing")?;

    let mut out = Vec::new();
    for (symbol, fytoken_v) in valid {
        let fytoken = fytoken_v.as_str().ok_or("fytoken not a string")?;
        if fytoken.len() < 11 {
            return Err(format!("fytoken too short: {fytoken}").into());
        }
        let ex_sg = &fytoken[..4];
        let segment = exch_seg_dict
            .get(ex_sg)
            .ok_or_else(|| format!("unknown exch_seg {ex_sg}"))?;
        let exch_token = &fytoken[10..];

        let is_index = symbol.ends_with("-INDEX");
        let prefix = if is_index { "if" } else { "sf" };
        out.push(format!("{prefix}|{segment}|{exch_token}"));
    }
    Ok(out)
}

fn strip_appid_prefix(token: &str) -> &str {
    match token.find(':') {
        Some(idx) => &token[idx + 1..],
        None => token,
    }
}

fn warn_if_token_kind_wrong(jwt: &str) {
    let Some(payload_b64) = jwt.split('.').nth(1) else {
        return;
    };
    let Ok(payload_bytes) = URL_SAFE_NO_PAD.decode(payload_b64) else {
        return;
    };
    let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&payload_bytes) else {
        return;
    };
    let sub = payload.get("sub").and_then(|v| v.as_str()).unwrap_or("?");
    if sub != "access_token" {
        eprintln!(
            "WARNING: FYERS_ACCESS_TOKEN has sub=\"{sub}\" but the symbol-token API requires sub=\"access_token\".\n\
             Did you skip the validate-authcode step? Run `cargo run --example auth_generate_access_token`\n\
             after setting FYERS_AUTH_CODE and FYERS_SECRET_KEY in .env."
        );
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

/// Build a channel-bitmap message used by full_mode (0x0c) and channel_resume (0x08).
/// Mirrors __full_mode_msg / __channel_resume_msg in the Python SDK.
fn build_channel_bitmap_message(req_type: u8, channel: u8) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.extend_from_slice(&0u16.to_be_bytes());

    buf.push(req_type);

    let field_count = if req_type == REQ_FULL_MODE { 2 } else { 1 };
    buf.push(field_count);

    let mut bitmap: u64 = 0;
    if (1..64).contains(&channel) {
        bitmap |= 1u64 << channel;
    }
    push_field(&mut buf, 1, &bitmap.to_be_bytes());

    if req_type == REQ_FULL_MODE {
        push_field(&mut buf, 2, &[70u8]);
    }

    buf
}

fn build_subscribe_message(
    hsm_symbols: &[String],
    channel_num: u8,
    access_token: &str,
    source: &str,
) -> Vec<u8> {
    let mut scrips_data = Vec::new();
    let count = u16::try_from(hsm_symbols.len()).expect("under 64Ki symbols");
    scrips_data.extend_from_slice(&count.to_be_bytes());
    for s in hsm_symbols {
        let bytes = s.as_bytes();
        scrips_data.push(u8::try_from(bytes.len()).expect("symbol under 256 bytes"));
        scrips_data.extend_from_slice(bytes);
    }

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

fn print_envelope(label: &str, env: &Envelope) {
    println!(
        "{label} envelope: req_type=0x{:02x} ({}) fields={}",
        env.req_type,
        req_type_name(env.req_type),
        env.fields.len()
    );
    for (idx, f) in env.fields.iter().enumerate() {
        println!("       field[{idx}] id={} len={} {}", f.id, f.value.len(), preview(&f.value));
    }
}

fn req_type_name(t: u8) -> &'static str {
    match t {
        0x01 => "auth",
        0x04 => "subscribe",
        0x05 => "unsubscribe",
        0x06 => "datafeed",
        0x07 => "channel_pause",
        0x08 => "channel_resume",
        0x0c => "full_mode",
        0x0d => "channel_buffer",
        _ => "unknown",
    }
}

fn preview(value: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(value) {
        if !s.is_empty() && s.chars().all(|c| !c.is_control() || c == '\n' || c == '\t') {
            let truncated: String = s.chars().take(80).collect();
            return format!(
                "text=\"{}{}\"",
                truncated,
                if s.len() > 80 { "\u{2026}" } else { "" }
            );
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

fn save(path: &str, data: &[u8]) {
    if let Err(e) = std::fs::write(path, data) {
        eprintln!("warn: failed to save {path}: {e}");
    }
}
