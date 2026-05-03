//! Binary wire protocol for the Fyers V3 market-data socket.
//!
//! The socket at `wss://socket.fyers.in/hsm/v1-5/prod` uses framed binary
//! messages (no JSON). Every request and control response is wrapped in the
//! same envelope:
//!
//! ```text
//! [2B BE u16: data_len] [1B: req_type] [1B: field_count]
//! { [1B: field_id] [2B BE u16: field_len] [field_len bytes] } * field_count
//! ```
//!
//! Datafeed frames (`req_type = 0x06`) carry one or more per-scrip payloads
//! after a 4-byte message number and 2-byte scrip count. Each scrip payload
//! is one of:
//!
//! - `0x53` snapshot: a fresh dump of the documented `data_val` fields, each
//!   encoded as a big-endian `i32`. Prices are scaled ×100.
//! - `0x55` update: same encoding as snapshot, but only the changed fields.
//! - `0x4c` lite: a smaller per-scrip payload used in lite mode.
//!
//! All builders here mirror the reference Python SDK
//! (`fyers_apiv3.FyersWebsocket.data_ws`) byte-for-byte.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use crate::error::{FyersError, Result};
use crate::models::ws::SymbolUpdate;

/// `req_type` byte values used on requests we send.
pub mod req_type {
    pub const AUTH: u8 = 0x01;
    pub const ACK: u8 = 0x03;
    pub const SUBSCRIBE: u8 = 0x04;
    pub const UNSUBSCRIBE: u8 = 0x05;
    pub const DATAFEED: u8 = 0x06;
    pub const CHANNEL_PAUSE: u8 = 0x07;
    pub const CHANNEL_RESUME: u8 = 0x08;
    pub const FULL_MODE: u8 = 0x0c;
    pub const CHANNEL_BUFFER: u8 = 0x0d;
}

/// Per-scrip data type byte at the start of a datafeed payload.
pub mod data_type {
    pub const SNAPSHOT: u8 = 0x53;
    pub const UPDATE: u8 = 0x55;
    pub const LITE: u8 = 0x4c;
    pub const FULL_HEADER: u8 = 0x46;
}

/// Channel mode marker carried in the auth message.
pub mod mode {
    pub const FULL: u8 = b'P';
    pub const LITE: u8 = b'L';
}

/// Documented fields emitted in snapshot/update payloads, in order.
///
/// Source: `fyers_apiv3/FyersWebsocket/map.json` `data_val`. The server
/// echoes the same ordering in the per-session schema dictionary returned
/// inside the auth response — see `auth_response_schema.json` for the full
/// definition. Prices are encoded as `i32 * 100`.
pub const DATA_VAL_FIELDS: &[&str] = &[
    "ltp",
    "vol_traded_today",
    "last_traded_time",
    "exch_feed_time",
    "bid_size",
    "ask_size",
    "bid_price",
    "ask_price",
    "last_traded_qty",
    "tot_buy_qty",
    "tot_sell_qty",
    "avg_trade_price",
    "OI",
    "low_price",
    "high_price",
    "Yhigh",
    "Ylow",
    "lower_ckt",
    "upper_ckt",
    "open_price",
    "prev_close_price",
];

/// Field identifiers that index into the price-scaled subset (×100 → rupees).
fn is_price_field(name: &str) -> bool {
    matches!(
        name,
        "ltp"
            | "bid_price"
            | "ask_price"
            | "avg_trade_price"
            | "low_price"
            | "high_price"
            | "Yhigh"
            | "Ylow"
            | "lower_ckt"
            | "upper_ckt"
            | "open_price"
            | "prev_close_price"
    )
}

/// Extract `hsm_key` from an APIv3 access-token JWT payload.
pub fn extract_hsm_key(access_token: &str) -> Result<String> {
    let jwt = strip_appid_prefix(access_token);
    let mut parts = jwt.split('.');
    let _header = parts.next().ok_or_else(|| invalid_token("missing header"))?;
    let payload_b64 = parts.next().ok_or_else(|| invalid_token("missing payload"))?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|err| invalid_token(format!("base64: {err}")))?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|err| invalid_token(format!("payload JSON: {err}")))?;
    payload
        .get("hsm_key")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| invalid_token("payload has no hsm_key"))
}

fn strip_appid_prefix(token: &str) -> &str {
    match token.find(':') {
        Some(idx) => &token[idx + 1..],
        None => token,
    }
}

fn invalid_token(detail: impl AsRef<str>) -> FyersError {
    FyersError::Validation(format!("invalid Fyers access token: {}", detail.as_ref()))
}

/// Build the documented binary auth frame.
pub fn build_auth_message(hsm_token: &str, channel_mode: u8, source: &str) -> Vec<u8> {
    let total = 18 + hsm_token.len() + source.len();
    let mut buf = Vec::with_capacity(total);
    let length_prefix = u16::try_from(total - 2).expect("auth message under 64KiB");
    buf.extend_from_slice(&length_prefix.to_be_bytes());
    buf.push(req_type::AUTH);
    buf.push(0x04);
    push_field(&mut buf, 1, hsm_token.as_bytes());
    push_field(&mut buf, 2, &[channel_mode]);
    push_field(&mut buf, 3, &[0x01]);
    push_field(&mut buf, 4, source.as_bytes());
    buf
}

/// Build a channel-bitmap message used by full-mode and channel-resume requests.
pub fn build_channel_bitmap_message(req: u8, channel_num: u8) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&0u16.to_be_bytes());
    buf.push(req);
    let field_count = if req == req_type::FULL_MODE { 2 } else { 1 };
    buf.push(field_count);

    let mut bitmap: u64 = 0;
    if (1..64).contains(&channel_num) {
        bitmap |= 1u64 << channel_num;
    }
    push_field(&mut buf, 1, &bitmap.to_be_bytes());

    if req == req_type::FULL_MODE {
        push_field(&mut buf, 2, &[data_type::FULL_HEADER]);
    }
    buf
}

/// Build the documented binary subscribe frame for the given HSM symbols.
pub fn build_subscribe_message(
    hsm_symbols: &[String],
    channel_num: u8,
    access_token: &str,
    source: &str,
) -> Vec<u8> {
    build_subscription_like(req_type::SUBSCRIBE, hsm_symbols, channel_num, access_token, source)
}

/// Build the documented binary unsubscribe frame for the given HSM symbols.
pub fn build_unsubscribe_message(
    hsm_symbols: &[String],
    channel_num: u8,
    access_token: &str,
    source: &str,
) -> Vec<u8> {
    build_subscription_like(req_type::UNSUBSCRIBE, hsm_symbols, channel_num, access_token, source)
}

fn build_subscription_like(
    req: u8,
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
    buf.push(req);
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

/// Parsed control envelope. Used for auth/subscribe/full-mode/resume responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope<'a> {
    pub req_type: u8,
    pub fields: Vec<EnvelopeField<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeField<'a> {
    pub id: u8,
    pub value: &'a [u8],
}

impl<'a> Envelope<'a> {
    pub fn field(&self, id: u8) -> Option<&'a [u8]> {
        self.fields.iter().find(|f| f.id == id).map(|f| f.value)
    }

    pub fn status_text(&self) -> Option<&'a str> {
        self.field(1).and_then(|v| std::str::from_utf8(v).ok())
    }

    pub fn is_ok(&self) -> bool {
        self.status_text() == Some("K")
    }
}

/// Parse the standard control envelope.
pub fn parse_envelope(data: &[u8]) -> Result<Envelope<'_>> {
    if data.len() < 4 {
        return Err(envelope_err(format!("frame too short: {} bytes", data.len())));
    }
    let req = data[2];
    let field_count = data[3] as usize;
    let mut offset = 4;
    let mut fields = Vec::with_capacity(field_count);
    for i in 0..field_count {
        if offset + 3 > data.len() {
            return Err(envelope_err(format!("truncated at field {i} header")));
        }
        let id = data[offset];
        let len = u16::from_be_bytes([data[offset + 1], data[offset + 2]]) as usize;
        offset += 3;
        if offset + len > data.len() {
            return Err(envelope_err(format!(
                "field {i} length {len} exceeds frame ({} bytes)",
                data.len()
            )));
        }
        fields.push(EnvelopeField {
            id,
            value: &data[offset..offset + len],
        });
        offset += len;
    }
    Ok(Envelope { req_type: req, fields })
}

fn envelope_err(detail: impl AsRef<str>) -> FyersError {
    FyersError::Validation(format!("data-socket envelope: {}", detail.as_ref()))
}

/// One per-scrip payload extracted from a datafeed frame.
#[derive(Debug, Clone, PartialEq)]
pub struct ScripFeed<'a> {
    pub data_type: u8,
    pub topic_id: u16,
    pub topic_name: &'a str,
    pub field_values: Vec<i32>,
}

/// Parse a datafeed frame (`req_type = 0x06`) into per-scrip payloads.
pub fn parse_datafeed(data: &[u8]) -> Result<Vec<ScripFeed<'_>>> {
    if data.len() < 9 {
        return Err(envelope_err("datafeed frame under 9 bytes"));
    }
    let scrip_count = u16::from_be_bytes([data[7], data[8]]) as usize;
    let mut offset = 9;
    let mut feeds = Vec::with_capacity(scrip_count);
    for s in 0..scrip_count {
        if offset + 4 > data.len() {
            return Err(envelope_err(format!("datafeed scrip {s}: truncated header")));
        }
        let dtype = data[offset];
        offset += 1;
        let topic_id = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        let name_len = data[offset] as usize;
        offset += 1;
        if offset + name_len + 1 > data.len() {
            return Err(envelope_err(format!("datafeed scrip {s}: truncated name")));
        }
        let topic_name = std::str::from_utf8(&data[offset..offset + name_len]).map_err(|err| {
            envelope_err(format!("datafeed scrip {s}: non-UTF8 topic name: {err}"))
        })?;
        offset += name_len;
        let field_count = data[offset] as usize;
        offset += 1;
        if offset + field_count * 4 > data.len() {
            return Err(envelope_err(format!(
                "datafeed scrip {s}: declared {field_count} i32 fields exceed frame"
            )));
        }
        let mut field_values = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let v = i32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            field_values.push(v);
        }
        feeds.push(ScripFeed {
            data_type: dtype,
            topic_id,
            topic_name,
            field_values,
        });
    }
    Ok(feeds)
}

/// Convert a parsed `ScripFeed` of `data_val`-ordered i32 values into a typed
/// [`SymbolUpdate`] event.
pub fn symbol_update_from_feed(feed: &ScripFeed<'_>) -> SymbolUpdate {
    let mut update = SymbolUpdate {
        event_type: match feed.data_type {
            data_type::SNAPSHOT => "sf".to_owned(),
            data_type::UPDATE => "sf".to_owned(),
            other => format!("0x{other:02x}"),
        },
        symbol: ticker_from_topic(feed.topic_name),
        ltp: 0.0,
        prev_close_price: None,
        high_price: None,
        low_price: None,
        open_price: None,
        ch: None,
        chp: None,
        vol_traded_today: None,
        last_traded_time: None,
        exch_feed_time: None,
        bid_size: None,
        ask_size: None,
        bid_price: None,
        ask_price: None,
        last_traded_qty: None,
        tot_buy_qty: None,
        tot_sell_qty: None,
        avg_trade_price: None,
    };

    for (i, &raw) in feed.field_values.iter().enumerate() {
        let Some(name) = DATA_VAL_FIELDS.get(i).copied() else {
            continue;
        };
        let scaled = if is_price_field(name) {
            f64::from(raw) / 100.0
        } else {
            f64::from(raw)
        };
        match name {
            "ltp" => update.ltp = scaled,
            "prev_close_price" => update.prev_close_price = Some(scaled),
            "high_price" => update.high_price = Some(scaled),
            "low_price" => update.low_price = Some(scaled),
            "open_price" => update.open_price = Some(scaled),
            "vol_traded_today" => update.vol_traded_today = Some(i64::from(raw)),
            "last_traded_time" => update.last_traded_time = Some(i64::from(raw)),
            "exch_feed_time" => update.exch_feed_time = Some(i64::from(raw)),
            "bid_size" => update.bid_size = Some(i64::from(raw)),
            "ask_size" => update.ask_size = Some(i64::from(raw)),
            "bid_price" => update.bid_price = Some(scaled),
            "ask_price" => update.ask_price = Some(scaled),
            "last_traded_qty" => update.last_traded_qty = Some(i64::from(raw)),
            "tot_buy_qty" => update.tot_buy_qty = Some(i64::from(raw)),
            "tot_sell_qty" => update.tot_sell_qty = Some(i64::from(raw)),
            "avg_trade_price" => update.avg_trade_price = Some(scaled),
            _ => {}
        }
    }

    if let (Some(ltp), Some(prev)) = (Some(update.ltp), update.prev_close_price) {
        let ch = ltp - prev;
        update.ch = Some(round2(ch));
        if prev != 0.0 {
            update.chp = Some(round2(ch / prev * 100.0));
        }
    }

    update
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Recover the human-readable ticker from an HSM topic name like `sf|nse_cm|3045`.
///
/// The server only sends the HSM-encoded form on the wire. Callers that want
/// the original `NSE:SBIN-EQ` string should look it up in their own
/// HSM-symbol → request-symbol map; this helper is a fallback that just returns
/// the HSM topic verbatim.
pub fn ticker_from_topic(topic: &str) -> String {
    topic.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_SBIN_SNAPSHOT: &[u8] =
        include_bytes!("../../fixtures/ws/data/captured/snapshot_NSE_SBIN-EQ.bin");

    #[test]
    fn auth_message_layout_matches_python_sdk_for_known_inputs() {
        // Synthetic 56-char hsm_token + "fyers-rs/0.1.0" source. Matches what
        // we produced in the M1 PoC exactly (88 bytes total).
        let hsm = "5defeeb4dda186d360dcf3cfd35216088f49ef5d4fc1b09aaccdc374";
        let bytes = build_auth_message(hsm, mode::FULL, "fyers-rs/0.1.0");
        assert_eq!(bytes.len(), 88);
        assert_eq!(&bytes[0..2], &[0x00, 0x56]); // 86 == total - 2
        assert_eq!(bytes[2], req_type::AUTH);
        assert_eq!(bytes[3], 0x04);
        // Field 1 = hsm token
        assert_eq!(bytes[4], 1);
        assert_eq!(&bytes[5..7], &(hsm.len() as u16).to_be_bytes());
        assert_eq!(&bytes[7..7 + hsm.len()], hsm.as_bytes());
        // Field 2 = mode 'P'
        let off = 7 + hsm.len();
        assert_eq!(bytes[off], 2);
        assert_eq!(&bytes[off + 1..off + 3], &[0x00, 0x01]);
        assert_eq!(bytes[off + 3], mode::FULL);
    }

    #[test]
    fn channel_bitmap_for_channel_11_full_mode_uses_bit_11() {
        let bytes = build_channel_bitmap_message(req_type::FULL_MODE, 11);
        assert_eq!(bytes[2], req_type::FULL_MODE);
        assert_eq!(bytes[3], 2); // field count
        // Field 1 = bitmap (8 bytes BE u64) with bit 11 set
        assert_eq!(bytes[4], 1);
        assert_eq!(&bytes[5..7], &[0x00, 0x08]);
        let bitmap = u64::from_be_bytes([
            bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
        ]);
        assert_eq!(bitmap, 1u64 << 11);
        // Field 2 = mode marker 0x46 ('F'/full header)
        assert_eq!(bytes[15], 2);
        assert_eq!(&bytes[16..18], &[0x00, 0x01]);
        assert_eq!(bytes[18], data_type::FULL_HEADER);
    }

    #[test]
    fn channel_resume_has_only_bitmap_field() {
        let bytes = build_channel_bitmap_message(req_type::CHANNEL_RESUME, 11);
        assert_eq!(bytes[2], req_type::CHANNEL_RESUME);
        assert_eq!(bytes[3], 1);
        assert_eq!(bytes.len(), 15); // 4 header + 1 fid + 2 len + 8 bitmap
    }

    #[test]
    fn subscribe_layout_for_one_hsm_symbol() {
        let symbols = vec!["sf|nse_cm|3045".to_owned()];
        let bytes = build_subscribe_message(&symbols, 11, "TOK", "src");
        assert_eq!(bytes[2], req_type::SUBSCRIBE);
        assert_eq!(bytes[3], 0x02);
        // field 1 starts at byte 4
        assert_eq!(bytes[4], 1);
        let field1_len = u16::from_be_bytes([bytes[5], bytes[6]]) as usize;
        // 2 bytes count + 1 byte len + 14 bytes name = 17 bytes
        assert_eq!(field1_len, 17);
        assert_eq!(&bytes[7..9], &[0x00, 0x01]); // count = 1
        assert_eq!(bytes[9], 14); // name length
        assert_eq!(&bytes[10..24], b"sf|nse_cm|3045");
        // field 2 starts after that
        let off = 7 + field1_len;
        assert_eq!(bytes[off], 2);
        assert_eq!(&bytes[off + 1..off + 3], &[0x00, 0x01]);
        assert_eq!(bytes[off + 3], 11); // channel num
    }

    #[test]
    fn parse_envelope_round_trips_subscribe_ack_shape() {
        // Mock: data_len=11, req_type=4, fc=2, field 1 "K", field 2 = 0xffff
        let bytes = [
            0x00, 0x0b, 0x04, 0x02, // header
            0x01, 0x00, 0x01, b'K', // field 1
            0x02, 0x00, 0x02, 0xff, 0xff, // field 2
        ];
        let env = parse_envelope(&bytes).unwrap();
        assert_eq!(env.req_type, req_type::SUBSCRIBE);
        assert_eq!(env.fields.len(), 2);
        assert!(env.is_ok());
        assert_eq!(env.field(2), Some(&[0xff, 0xff][..]));
    }

    #[test]
    fn parse_envelope_rejects_truncated_field() {
        // Declares field length 100 but only 4 bytes follow.
        let bytes = [0x00, 0x05, 0x04, 0x01, 0x01, 0x00, 0x64, 0x00];
        let err = parse_envelope(&bytes).unwrap_err();
        assert!(format!("{err}").contains("exceeds frame"));
    }

    #[test]
    fn parse_datafeed_decodes_captured_sbin_snapshot() {
        let feeds = parse_datafeed(FIXTURE_SBIN_SNAPSHOT).expect("decode snapshot");
        assert_eq!(feeds.len(), 1, "one scrip in fixture");
        let feed = &feeds[0];
        assert_eq!(feed.data_type, data_type::SNAPSHOT);
        assert_eq!(feed.topic_name, "sf|nse_cm|3045");
        assert_eq!(feed.field_values.len(), 21);
        assert_eq!(feed.field_values[0], 106845, "ltp raw = 106845 (=1068.45)");
        assert_eq!(feed.field_values[20], 108690, "prev_close_price raw");
    }

    #[test]
    fn symbol_update_from_captured_sbin_snapshot_has_real_prices() {
        let feeds = parse_datafeed(FIXTURE_SBIN_SNAPSHOT).expect("decode snapshot");
        let update = symbol_update_from_feed(&feeds[0]);
        assert_eq!(update.symbol, "sf|nse_cm|3045");
        assert!((update.ltp - 1068.45).abs() < f64::EPSILON);
        assert_eq!(update.prev_close_price, Some(1086.90));
        assert_eq!(update.open_price, Some(1075.00));
        assert_eq!(update.high_price, Some(1077.70));
        assert_eq!(update.low_price, Some(1063.00));
        assert_eq!(update.bid_price, Some(1068.45));
        assert_eq!(update.bid_size, Some(14914));
        assert_eq!(update.ask_size, Some(0));
        // Derived ch / chp from ltp vs prev_close_price.
        assert!(update.ch.unwrap() < 0.0);
        assert!(update.chp.unwrap() < 0.0);
    }

    #[test]
    fn extract_hsm_key_pulls_claim_from_jwt() {
        // Synthesize a minimal JWT with hsm_key in the payload. Header and
        // signature are placeholders — the function ignores them.
        let header = URL_SAFE_NO_PAD.encode(b"{\"alg\":\"none\"}");
        let payload =
            URL_SAFE_NO_PAD.encode(b"{\"sub\":\"access_token\",\"hsm_key\":\"deadbeef\"}");
        let jwt = format!("{header}.{payload}.sig");
        assert_eq!(extract_hsm_key(&jwt).unwrap(), "deadbeef");
        // APPID:JWT prefix is stripped.
        let prefixed = format!("APPID-100:{jwt}");
        assert_eq!(extract_hsm_key(&prefixed).unwrap(), "deadbeef");
    }

    #[test]
    fn extract_hsm_key_rejects_token_without_claim() {
        let header = URL_SAFE_NO_PAD.encode(b"{\"alg\":\"none\"}");
        let payload = URL_SAFE_NO_PAD.encode(b"{\"sub\":\"auth_code\"}");
        let jwt = format!("{header}.{payload}.sig");
        let err = extract_hsm_key(&jwt).unwrap_err();
        assert!(format!("{err}").contains("no hsm_key"));
    }
}
