//! Encoding extensions for Snowflake ID (u64): base62, base64, url-safe base64
//! Enabled via features: "base62", "base64"

/// ID 编码扩展 Trait
pub trait IdEncoding {
    /// Encode to base62 (short, URL-safe, human-friendly)
    #[cfg(feature = "base62")]
    fn to_base62(&self) -> String;

    /// Encode to standard base64
    #[cfg(feature = "base64")]
    fn to_base64(&self) -> String;

    /// Encode to URL-safe base64 (no padding, no /+)
    #[cfg(feature = "base64")]
    fn to_base64_url(&self) -> String;
}

/// ID 解码扩展 Trait
pub trait IdDecoding: Sized {
    /// Decode from base62 string
    #[cfg(feature = "base62")]
    fn from_base62(s: &str) -> Result<u64, Box<dyn std::error::Error>>;

    /// Decode from standard base64 string
    #[cfg(feature = "base64")]
    fn from_base64(s: &str) -> Result<u64, Box<dyn std::error::Error>>;

    /// Decode from URL-safe base64 string
    #[cfg(feature = "base64")]
    fn from_base64_url(s: &str) -> Result<u64, Box<dyn std::error::Error>>;
}

impl IdEncoding for u64 {
    #[cfg(feature = "base62")]
    fn to_base62(&self) -> String {
        base62::encode(*self)
    }

    #[cfg(feature = "base64")]
    fn to_base64(&self) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine};
        STANDARD.encode(self.to_be_bytes())
    }

    #[cfg(feature = "base64")]
    fn to_base64_url(&self) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        URL_SAFE_NO_PAD.encode(self.to_be_bytes())
    }
}

impl IdDecoding for u64 {
    #[cfg(feature = "base62")]
    fn from_base62(s: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let decoded: u128 = base62::decode(s)?;
        decoded.try_into().map_err(|_| "value too large for u64".into())
    }

    #[cfg(feature = "base64")]
    fn from_base64(s: &str) -> Result<u64, Box<dyn std::error::Error>> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        let bytes = STANDARD.decode(s)?;
        let arr: [u8; 8] = bytes.try_into().map_err(|_| "invalid base64 length for u64")?;
        Ok(u64::from_be_bytes(arr))
    }

    #[cfg(feature = "base64")]
    fn from_base64_url(s: &str) -> Result<u64, Box<dyn std::error::Error>> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let bytes = URL_SAFE_NO_PAD.decode(s)?;
        let arr: [u8; 8] = bytes.try_into().map_err(|_| "invalid base64 url length for u64")?;
        Ok(u64::from_be_bytes(arr))
    }
}

// ------------------------------ TESTS ------------------------------
#[cfg(any(feature = "base62", feature = "base64"))]
#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ID: u64 = 1234567890123456789;

    #[test]
    #[cfg(feature = "base62")]
    fn test_base62_encode_decode() {
        let enc = TEST_ID.to_base62();
        let dec = u64::from_base62(&enc).unwrap();
        assert_eq!(dec, TEST_ID);
    }

    #[test]
    #[cfg(feature = "base64")]
    fn test_base64_encode_decode() {
        let enc = TEST_ID.to_base64();
        let dec = u64::from_base64(&enc).unwrap();
        assert_eq!(dec, TEST_ID);
    }

    #[test]
    #[cfg(feature = "base64")]
    fn test_base64_url_encode_decode() {
        let enc = TEST_ID.to_base64_url();
        let dec = u64::from_base64_url(&enc).unwrap();
        assert_eq!(dec, TEST_ID);
    }
}