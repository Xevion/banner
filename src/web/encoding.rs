//! Content encoding negotiation for pre-compressed asset serving.
//!
//! Parses Accept-Encoding headers with quality values and returns
//! supported encodings in priority order for content negotiation.

use axum::http::{HeaderMap, HeaderValue, header};

/// Minimum size threshold for compression (bytes).
///
/// Must match `MIN_SIZE` in `web/scripts/compress-assets.ts`.
pub const COMPRESSION_MIN_SIZE: usize = 512;

/// Supported content encodings in priority order (best compression first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentEncoding {
    Zstd,
    Brotli,
    Gzip,
    Identity,
}

impl ContentEncoding {
    /// File extension suffix for pre-compressed variant lookup.
    #[inline]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Zstd => ".zst",
            Self::Brotli => ".br",
            Self::Gzip => ".gz",
            Self::Identity => "",
        }
    }

    /// `Content-Encoding` header value, or `None` for identity.
    #[inline]
    pub fn header_value(&self) -> Option<HeaderValue> {
        match self {
            Self::Zstd => Some(HeaderValue::from_static("zstd")),
            Self::Brotli => Some(HeaderValue::from_static("br")),
            Self::Gzip => Some(HeaderValue::from_static("gzip")),
            Self::Identity => None,
        }
    }

    /// Default priority when quality values are equal (higher = better).
    #[inline]
    fn default_priority(&self) -> u8 {
        match self {
            Self::Zstd => 4,
            Self::Brotli => 3,
            Self::Gzip => 2,
            Self::Identity => 1,
        }
    }
}

/// Parse `Accept-Encoding` header and return supported encodings in priority order.
///
/// Supports quality values: `Accept-Encoding: gzip;q=0.8, br;q=1.0, zstd`
/// When quality values are equal: zstd > brotli > gzip > identity.
/// Encodings with `q=0` are excluded.
pub fn parse_accepted_encodings(headers: &HeaderMap) -> Vec<ContentEncoding> {
    let Some(accept) = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
    else {
        return vec![ContentEncoding::Identity];
    };

    let mut encodings: Vec<(ContentEncoding, f32)> = Vec::new();

    for part in accept.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (encoding_str, quality) = if let Some((enc, params)) = part.split_once(';') {
            let q = params
                .split(';')
                .find_map(|p| p.trim().strip_prefix("q="))
                .and_then(|q| q.parse::<f32>().ok())
                .unwrap_or(1.0);
            (enc.trim(), q)
        } else {
            (part, 1.0)
        };

        if quality == 0.0 {
            continue;
        }

        let encoding = match encoding_str.to_lowercase().as_str() {
            "zstd" => ContentEncoding::Zstd,
            "br" | "brotli" => ContentEncoding::Brotli,
            "gzip" | "x-gzip" => ContentEncoding::Gzip,
            "*" => ContentEncoding::Gzip,
            "identity" => ContentEncoding::Identity,
            _ => continue,
        };

        encodings.push((encoding, quality));
    }

    // Sort by quality (desc), then default priority (desc)
    encodings.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.0.default_priority().cmp(&a.0.default_priority()))
    });

    if encodings.is_empty() {
        vec![ContentEncoding::Identity]
    } else {
        encodings.into_iter().map(|(e, _)| e).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_all_encodings() {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, "gzip, br, zstd".parse().unwrap());
        let encodings = parse_accepted_encodings(&headers);
        assert_eq!(encodings[0], ContentEncoding::Zstd);
        assert_eq!(encodings[1], ContentEncoding::Brotli);
        assert_eq!(encodings[2], ContentEncoding::Gzip);
    }

    #[test]
    fn test_parse_with_quality_values() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_ENCODING,
            "gzip;q=1.0, br;q=0.5, zstd;q=0.8".parse().unwrap(),
        );
        let encodings = parse_accepted_encodings(&headers);
        assert_eq!(encodings[0], ContentEncoding::Gzip);
        assert_eq!(encodings[1], ContentEncoding::Zstd);
        assert_eq!(encodings[2], ContentEncoding::Brotli);
    }

    #[test]
    fn test_no_header_returns_identity() {
        let headers = HeaderMap::new();
        let encodings = parse_accepted_encodings(&headers);
        assert_eq!(encodings, vec![ContentEncoding::Identity]);
    }

    #[test]
    fn test_disabled_encoding_excluded() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_ENCODING,
            "zstd;q=0, br, gzip".parse().unwrap(),
        );
        let encodings = parse_accepted_encodings(&headers);
        assert_eq!(encodings[0], ContentEncoding::Brotli);
        assert_eq!(encodings[1], ContentEncoding::Gzip);
        assert!(!encodings.contains(&ContentEncoding::Zstd));
    }

    #[test]
    fn test_real_chrome_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT_ENCODING,
            "gzip, deflate, br, zstd".parse().unwrap(),
        );
        assert_eq!(parse_accepted_encodings(&headers)[0], ContentEncoding::Zstd);
    }

    #[test]
    fn test_extensions() {
        assert_eq!(ContentEncoding::Zstd.extension(), ".zst");
        assert_eq!(ContentEncoding::Brotli.extension(), ".br");
        assert_eq!(ContentEncoding::Gzip.extension(), ".gz");
        assert_eq!(ContentEncoding::Identity.extension(), "");
    }

    #[test]
    fn test_header_values() {
        assert_eq!(
            ContentEncoding::Zstd.header_value().unwrap(),
            HeaderValue::from_static("zstd")
        );
        assert_eq!(
            ContentEncoding::Brotli.header_value().unwrap(),
            HeaderValue::from_static("br")
        );
        assert!(ContentEncoding::Identity.header_value().is_none());
    }
}
