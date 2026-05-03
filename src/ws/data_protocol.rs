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
use crate::models::ws::{DepthUpdate, IndexUpdate, SymbolUpdate};

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

/// Channel mode marker carried in the auth message and full/lite mode set messages.
pub mod mode {
    /// Marker placed in the auth message field 2 to request full mode.
    pub const FULL: u8 = b'P';
    /// Marker placed in the auth message field 2 to request lite mode.
    pub const LITE: u8 = b'L';
    /// Marker emitted by the channel-mode set message to select full mode.
    pub const FULL_HEADER: u8 = 0x46;
    /// Marker emitted by the channel-mode set message to select lite mode.
    pub const LITE_HEADER: u8 = 0x4c;
}

/// Documented fields emitted in snapshot/update payloads in full mode.
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

/// Documented fields emitted in lite-mode payloads (`data_type = 0x4c`).
///
/// Source: `fyers_apiv3/FyersWebsocket/map.json` `lite_val`, dropping the
/// trailing `symbol`/`type` metadata which travel outside the i32 array.
pub const LITE_VAL_FIELDS: &[&str] = &["ltp"];

/// Documented fields emitted for index symbols (`if|...` topics).
///
/// Source: `map.json` `index_val`, less the trailing metadata.
pub const INDEX_VAL_FIELDS: &[&str] = &[
    "ltp",
    "prev_close_price",
    "exch_feed_time",
    "high_price",
    "low_price",
    "open_price",
];

/// Documented fields emitted for depth/L2 payloads (`dp|...` topics).
///
/// Source: `map.json` `depthvalue`, less the trailing metadata. 30 i32
/// fields covering 5 levels each of bid/ask price, size, and order count.
pub const DEPTH_VAL_FIELDS: &[&str] = &[
    "bid_price1",
    "bid_price2",
    "bid_price3",
    "bid_price4",
    "bid_price5",
    "ask_price1",
    "ask_price2",
    "ask_price3",
    "ask_price4",
    "ask_price5",
    "bid_size1",
    "bid_size2",
    "bid_size3",
    "bid_size4",
    "bid_size5",
    "ask_size1",
    "ask_size2",
    "ask_size3",
    "ask_size4",
    "ask_size5",
    "bid_order1",
    "bid_order2",
    "bid_order3",
    "bid_order4",
    "bid_order5",
    "ask_order1",
    "ask_order2",
    "ask_order3",
    "ask_order4",
    "ask_order5",
];

/// Field identifiers that index into the price-scaled subset (×100 → rupees).
fn is_price_field(name: &str) -> bool {
    if matches!(
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
    ) {
        return true;
    }
    name.starts_with("bid_price") || name.starts_with("ask_price")
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

/// Build a channel-bitmap message for the full-mode (`0x0c`) request, using
/// the documented full-mode marker (`0x46`).
pub fn build_channel_bitmap_message(req: u8, channel_num: u8) -> Vec<u8> {
    build_channel_bitmap_message_with_marker(req, channel_num, mode::FULL_HEADER)
}

/// Build a channel-bitmap message with an explicit mode marker, allowing the
/// caller to pick between full mode (`0x46`) and lite mode (`0x4c`).
pub fn build_channel_bitmap_message_with_marker(
    req: u8,
    channel_num: u8,
    mode_marker: u8,
) -> Vec<u8> {
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
        push_field(&mut buf, 2, &[mode_marker]);
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

/// Choose which named-field schema applies to a given feed.
///
/// `lite_val` is selected for any feed whose `data_type` byte is the
/// documented lite-mode marker, regardless of topic. Otherwise the topic
/// prefix decides: `sf|...` -> `data_val`, `if|...` -> `index_val`,
/// `dp|...` -> `depth_val`.
pub fn schema_for_feed(feed: &ScripFeed<'_>) -> &'static [&'static str] {
    if feed.data_type == data_type::LITE {
        return LITE_VAL_FIELDS;
    }
    match feed.topic_name.split('|').next() {
        Some("if") => INDEX_VAL_FIELDS,
        Some("dp") => DEPTH_VAL_FIELDS,
        _ => DATA_VAL_FIELDS,
    }
}

/// Convert a parsed `ScripFeed` of `data_val`-ordered i32 values into a typed
/// [`SymbolUpdate`] event. Use [`schema_for_feed`] to pick the schema; for
/// lite-mode feeds pass [`LITE_VAL_FIELDS`].
pub fn symbol_update_from_feed_with_schema(
    feed: &ScripFeed<'_>,
    schema: &[&str],
) -> SymbolUpdate {
    let mut update = SymbolUpdate {
        event_type: match feed.data_type {
            data_type::SNAPSHOT | data_type::UPDATE => "sf".to_owned(),
            data_type::LITE => "lit".to_owned(),
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
        let Some(name) = schema.get(i).copied() else {
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

    if update.prev_close_price.is_some() {
        let prev = update.prev_close_price.unwrap_or(0.0);
        if prev != 0.0 {
            let ch = update.ltp - prev;
            update.ch = Some(round2(ch));
            update.chp = Some(round2(ch / prev * 100.0));
        }
    }

    update
}

/// Backwards-compatible helper assuming the full-mode `data_val` schema.
pub fn symbol_update_from_feed(feed: &ScripFeed<'_>) -> SymbolUpdate {
    symbol_update_from_feed_with_schema(feed, schema_for_feed(feed))
}

/// Convert a parsed `ScripFeed` for an `if|...` topic into an [`IndexUpdate`].
pub fn index_update_from_feed(feed: &ScripFeed<'_>) -> IndexUpdate {
    let mut ltp = 0.0;
    let mut prev_close_price = 0.0;
    let mut high_price = 0.0;
    let mut low_price = 0.0;
    let mut open_price = 0.0;
    let mut exch_feed_time: Option<i64> = None;

    for (i, &raw) in feed.field_values.iter().enumerate() {
        let Some(name) = INDEX_VAL_FIELDS.get(i).copied() else {
            continue;
        };
        let scaled = if is_price_field(name) {
            f64::from(raw) / 100.0
        } else {
            f64::from(raw)
        };
        match name {
            "ltp" => ltp = scaled,
            "prev_close_price" => prev_close_price = scaled,
            "high_price" => high_price = scaled,
            "low_price" => low_price = scaled,
            "open_price" => open_price = scaled,
            "exch_feed_time" => exch_feed_time = Some(i64::from(raw)),
            _ => {}
        }
    }

    let (ch, chp) = if prev_close_price != 0.0 {
        let diff = ltp - prev_close_price;
        (
            round2(diff),
            round2(diff / prev_close_price * 100.0),
        )
    } else {
        (0.0, 0.0)
    };

    IndexUpdate {
        event_type: "if".to_owned(),
        symbol: ticker_from_topic(feed.topic_name),
        ltp,
        prev_close_price,
        high_price,
        low_price,
        open_price,
        ch,
        chp,
        exch_feed_time,
    }
}

/// Convert a parsed `ScripFeed` for a `dp|...` topic into a [`DepthUpdate`].
pub fn depth_update_from_feed(feed: &ScripFeed<'_>) -> DepthUpdate {
    let mut prices = [0.0_f64; 10];
    let mut sizes = [0_i64; 10];
    let mut orders = [0_i64; 10];

    for (i, &raw) in feed.field_values.iter().enumerate() {
        let Some(name) = DEPTH_VAL_FIELDS.get(i).copied() else {
            continue;
        };
        if let Some(level) = depth_level_index(name) {
            if name.starts_with("bid_price") {
                prices[level] = f64::from(raw) / 100.0;
            } else if name.starts_with("ask_price") {
                prices[5 + level] = f64::from(raw) / 100.0;
            } else if name.starts_with("bid_size") {
                sizes[level] = i64::from(raw);
            } else if name.starts_with("ask_size") {
                sizes[5 + level] = i64::from(raw);
            } else if name.starts_with("bid_order") {
                orders[level] = i64::from(raw);
            } else if name.starts_with("ask_order") {
                orders[5 + level] = i64::from(raw);
            }
        }
    }

    DepthUpdate {
        event_type: "dp".to_owned(),
        symbol: ticker_from_topic(feed.topic_name),
        bid_price1: prices[0],
        bid_price2: prices[1],
        bid_price3: prices[2],
        bid_price4: prices[3],
        bid_price5: prices[4],
        ask_price1: prices[5],
        ask_price2: prices[6],
        ask_price3: prices[7],
        ask_price4: prices[8],
        ask_price5: prices[9],
        bid_size1: sizes[0],
        bid_size2: sizes[1],
        bid_size3: sizes[2],
        bid_size4: sizes[3],
        bid_size5: sizes[4],
        ask_size1: sizes[5],
        ask_size2: sizes[6],
        ask_size3: sizes[7],
        ask_size4: sizes[8],
        ask_size5: sizes[9],
        bid_order1: orders[0],
        bid_order2: orders[1],
        bid_order3: orders[2],
        bid_order4: orders[3],
        bid_order5: orders[4],
        ask_order1: orders[5],
        ask_order2: orders[6],
        ask_order3: orders[7],
        ask_order4: orders[8],
        ask_order5: orders[9],
    }
}

fn depth_level_index(name: &str) -> Option<usize> {
    let suffix = name.chars().last()?;
    match suffix {
        '1' => Some(0),
        '2' => Some(1),
        '3' => Some(2),
        '4' => Some(3),
        '5' => Some(4),
        _ => None,
    }
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

    #[test]
    fn full_mode_marker_can_be_overridden_for_lite() {
        let bytes = build_channel_bitmap_message_with_marker(
            req_type::FULL_MODE,
            11,
            mode::LITE_HEADER,
        );
        // Field 2 marker byte sits 4 (header) + 11 (field 1: id+len+8B bitmap) = 15.
        assert_eq!(bytes[18], mode::LITE_HEADER);
    }

    #[test]
    fn schema_for_feed_picks_lite_for_lite_data_type() {
        let feed = ScripFeed {
            data_type: data_type::LITE,
            topic_id: 0,
            topic_name: "sf|nse_cm|3045",
            field_values: vec![100],
        };
        assert_eq!(schema_for_feed(&feed), LITE_VAL_FIELDS);
    }

    #[test]
    fn schema_for_feed_picks_index_for_if_topic() {
        let feed = ScripFeed {
            data_type: data_type::SNAPSHOT,
            topic_id: 0,
            topic_name: "if|nse_cm|26000",
            field_values: vec![],
        };
        assert_eq!(schema_for_feed(&feed), INDEX_VAL_FIELDS);
    }

    #[test]
    fn schema_for_feed_picks_depth_for_dp_topic() {
        let feed = ScripFeed {
            data_type: data_type::SNAPSHOT,
            topic_id: 0,
            topic_name: "dp|nse_cm|3045",
            field_values: vec![],
        };
        assert_eq!(schema_for_feed(&feed), DEPTH_VAL_FIELDS);
    }

    #[test]
    fn lite_mode_feed_only_populates_ltp_on_symbol_update() {
        let feed = ScripFeed {
            data_type: data_type::LITE,
            topic_id: 7,
            topic_name: "sf|nse_cm|3045",
            field_values: vec![50055], // 500.55 in scaled rupees
        };
        let update = symbol_update_from_feed(&feed);
        assert_eq!(update.event_type, "lit");
        assert!((update.ltp - 500.55).abs() < f64::EPSILON);
        assert!(update.prev_close_price.is_none());
        assert!(update.high_price.is_none());
        assert!(update.bid_price.is_none());
    }

    #[test]
    fn index_feed_with_synthetic_values_decodes_to_index_update() {
        // ltp=2618810, prev_close=2621605, ts=1727428424,
        // high=2627735, low=2616695, open=2624825 — Nifty50-shaped
        let feed = ScripFeed {
            data_type: data_type::SNAPSHOT,
            topic_id: 11,
            topic_name: "if|nse_cm|26000",
            field_values: vec![2_618_810, 2_621_605, 1_727_428_424, 2_627_735, 2_616_695, 2_624_825],
        };
        let update = index_update_from_feed(&feed);
        assert_eq!(update.symbol, "if|nse_cm|26000");
        assert!((update.ltp - 26188.10).abs() < f64::EPSILON);
        assert!((update.prev_close_price - 26216.05).abs() < f64::EPSILON);
        assert!((update.high_price - 26277.35).abs() < f64::EPSILON);
        assert!((update.low_price - 26166.95).abs() < f64::EPSILON);
        assert!((update.open_price - 26248.25).abs() < f64::EPSILON);
        assert_eq!(update.exch_feed_time, Some(1_727_428_424));
        assert!(update.ch < 0.0); // 26188.10 < 26216.05
        assert!(update.chp < 0.0);
    }

    #[test]
    fn depth_feed_with_synthetic_values_decodes_five_levels_each_side() {
        // 5 bid prices, 5 ask prices, 5 bid sizes, 5 ask sizes,
        // 5 bid orders, 5 ask orders. All scaled prices ×100.
        let mut values = Vec::with_capacity(30);
        // bids: 1068.45, 1068.40, 1068.35, 1068.30, 1068.25
        for i in 0..5_i32 {
            values.push(106_845 - 5 * i);
        }
        // asks: 1068.50, 1068.55, 1068.60, 1068.65, 1068.70
        for i in 0..5_i32 {
            values.push(106_850 + 5 * i);
        }
        // bid sizes 100..500
        for i in 0..5_i32 {
            values.push((i + 1) * 100);
        }
        // ask sizes 200..600
        for i in 0..5_i32 {
            values.push((i + 2) * 100);
        }
        // bid orders / ask orders
        for i in 0..5_i32 {
            values.push(i + 1);
        }
        for i in 0..5_i32 {
            values.push(i + 6);
        }

        let feed = ScripFeed {
            data_type: data_type::SNAPSHOT,
            topic_id: 99,
            topic_name: "dp|nse_cm|3045",
            field_values: values,
        };
        let depth = depth_update_from_feed(&feed);
        assert_eq!(depth.symbol, "dp|nse_cm|3045");
        assert!((depth.bid_price1 - 1068.45).abs() < f64::EPSILON);
        assert!((depth.bid_price5 - 1068.25).abs() < f64::EPSILON);
        assert!((depth.ask_price1 - 1068.50).abs() < f64::EPSILON);
        assert!((depth.ask_price5 - 1068.70).abs() < f64::EPSILON);
        assert_eq!(depth.bid_size1, 100);
        assert_eq!(depth.bid_size5, 500);
        assert_eq!(depth.ask_size1, 200);
        assert_eq!(depth.ask_size5, 600);
        assert_eq!(depth.bid_order1, 1);
        assert_eq!(depth.ask_order1, 6);
        assert_eq!(depth.bid_order5, 5);
        assert_eq!(depth.ask_order5, 10);
    }
}
