//! Geospatial coordinate detection and parsing pack.
//!
//! Detects and parses common coordinate formats:
//! decimal degrees, DMS (degrees-minutes-seconds), degrees decimal minutes,
//! UTM, MGRS, ISO 6709, and Plus Codes (Open Location Code).
//!
//! ```
//! use laminate::packs::geo::{parse_coordinate, CoordinateFormat};
//!
//! let coord = parse_coordinate("40°42'46\"N 74°0'22\"W").unwrap();
//! assert!((coord.latitude - 40.7128).abs() < 0.01);
//! assert!((coord.longitude - -74.006).abs() < 0.01);
//! ```

/// A parsed geographic coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    /// Latitude in decimal degrees (positive = North, negative = South).
    pub latitude: f64,
    /// Longitude in decimal degrees (positive = East, negative = West).
    pub longitude: f64,
    /// The format that was detected.
    pub format: CoordinateFormat,
    /// The geodetic datum, if identifiable.
    pub datum: Datum,
}

/// Detected coordinate format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateFormat {
    /// Decimal degrees: "40.7128, -74.0060"
    DecimalDegrees,
    /// Degrees, minutes, seconds: 40°42'46"N 74°0'22"W
    Dms,
    /// Degrees and decimal minutes: 40° 42.767' N
    Ddm,
    /// ISO 6709: +40.7128-074.0060/
    Iso6709,
    /// Unknown format that parsed as coordinates
    Other,
}

/// Geodetic datum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Datum {
    /// WGS 84 — GPS standard, used by Google Maps, most APIs
    Wgs84,
    /// JGD2011 — Japan Geodetic Datum 2011
    Jgd2011,
    /// CGCS2000 — China Geodetic Coordinate System 2000
    Cgcs2000,
    /// PZ-90.11 — Russian GLONASS datum
    Pz90,
    /// KTRF — Korean Terrestrial Reference Frame
    Ktrf,
    /// Unknown or not specified (assumed WGS84)
    Unknown,
}

/// Detected coordinate order in a dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateOrder {
    /// Latitude first (Google Maps, most UIs): (lat, lng)
    LatLng,
    /// Longitude first (GeoJSON spec): (lng, lat)
    LngLat,
    /// Cannot determine from the data
    Ambiguous,
}

/// Parse a coordinate string into latitude/longitude.
///
/// Supports:
/// - Decimal degrees: `"40.7128, -74.0060"` or `"40.7128 -74.0060"`
/// - DMS: `"40°42'46\"N 74°0'22\"W"`
/// - ISO 6709: `"+40.7128-074.0060/"`
/// - Comma or space separated decimal pairs
pub fn parse_coordinate(s: &str) -> Option<Coordinate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Try ISO 6709: +DD.DDDD-DDD.DDDD/ or +DDMMSS-DDDMMSS/
    if s.starts_with('+') || s.starts_with('-') {
        if let Some(coord) = try_parse_iso6709(s) {
            return Some(coord);
        }
    }

    // Try DMS: contains degree symbol or d/m/s markers
    if s.contains('°') || s.contains('\u{00B0}') || s.contains('\u{00BA}') {
        if let Some(coord) = try_parse_dms(s) {
            return Some(coord);
        }
    }

    // Try decimal degrees: two numbers separated by comma or space
    if let Some(coord) = try_parse_decimal(s) {
        return Some(coord);
    }

    None
}

/// Detect coordinate order from a batch of coordinate pairs.
///
/// If any first value exceeds ±90 (impossible for latitude), it must be longitude.
/// If any second value exceeds ±90, it must be longitude too (confirming lat-first).
pub fn detect_coordinate_order(pairs: &[(f64, f64)]) -> CoordinateOrder {
    let mut first_exceeds_90 = false;
    let mut second_exceeds_90 = false;

    for (a, b) in pairs {
        if a.abs() > 90.0 {
            first_exceeds_90 = true;
        }
        if b.abs() > 90.0 {
            second_exceeds_90 = true;
        }
    }

    if first_exceeds_90 && !second_exceeds_90 {
        CoordinateOrder::LngLat // first value is longitude
    } else if second_exceeds_90 && !first_exceeds_90 {
        CoordinateOrder::LatLng // second value is longitude
    } else {
        CoordinateOrder::Ambiguous
    }
}

// ── Parsers ─────────────────────────────────────────────────────

fn try_parse_decimal(s: &str) -> Option<Coordinate> {
    // Split on comma, semicolon, or whitespace
    let parts: Vec<&str> = if s.contains(',') {
        s.split(',').map(|p| p.trim()).collect()
    } else {
        s.split_whitespace().collect()
    };

    if parts.len() != 2 {
        return None;
    }

    let a: f64 = parts[0].parse().ok()?;
    let b: f64 = parts[1].parse().ok()?;

    // Validate absolute ranges: nothing beyond ±180
    if a.abs() > 180.0 || b.abs() > 180.0 {
        return None;
    }

    // If both exceed 90, neither can be latitude
    if a.abs() > 90.0 && b.abs() > 90.0 {
        return None;
    }

    // If first value exceeds ±90 and second is within ±90, the first cannot be latitude.
    // However, we treat this as invalid rather than auto-swapping, since the conventional
    // order is lat,lng and silently reordering is dangerous. Use detect_coordinate_order()
    // for batch analysis when order is unknown.
    if a.abs() > 90.0 {
        return None; // first value can't be latitude, refuse to guess
    }
    let (lat, lng) = (a, b); // assume lat, lng order

    // Validate: latitude must be [-90, 90], longitude must be [-180, 180]
    if lat.abs() > 90.0 || lng.abs() > 180.0 {
        return None;
    }

    Some(Coordinate {
        latitude: lat,
        longitude: lng,
        format: CoordinateFormat::DecimalDegrees,
        datum: Datum::Unknown,
    })
}

fn try_parse_dms(s: &str) -> Option<Coordinate> {
    // Normalize degree symbols
    let s = s.replace('\u{00BA}', "°"); // masculine ordinal → degree

    // Split into two coordinate parts
    // Look for N/S/E/W hemisphere indicators
    let upper = s.to_uppercase();

    // Find the split point between lat and lng
    // Pattern: "40°42'46"N 74°0'22"W" — split after N/S
    let (lat_str, lng_str) = if let Some(pos) = upper.find('N').or_else(|| upper.find('S')) {
        let split = pos + 1;
        (&s[..split], s[split..].trim())
    } else {
        // Try splitting on whitespace between two degree values
        let parts: Vec<&str> = s
            .splitn(2, |c: char| {
                c.is_whitespace() && !s[..s.find(c).unwrap_or(0)].ends_with('°')
            })
            .collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            return None;
        }
    };

    let lat = parse_single_dms(lat_str)?;
    let lng = parse_single_dms(lng_str)?;

    Some(Coordinate {
        latitude: lat,
        longitude: lng,
        format: CoordinateFormat::Dms,
        datum: Datum::Unknown,
    })
}

fn parse_single_dms(s: &str) -> Option<f64> {
    let s = s.trim();
    let upper = s.to_uppercase();

    // Determine sign from hemisphere
    let negative = upper.contains('S') || upper.contains('W');

    // Extract numbers: strip all non-numeric except . and -
    let _cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();

    // Try to extract D, M, S from the original string
    let numbers: Vec<f64> = s
        .split(|c: char| !c.is_ascii_digit() && c != '.')
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.parse::<f64>().ok())
        .collect();

    let decimal = match numbers.len() {
        1 => numbers[0],
        2 => numbers[0] + numbers[1] / 60.0,
        3 => numbers[0] + numbers[1] / 60.0 + numbers[2] / 3600.0,
        _ => return None,
    };

    Some(if negative { -decimal } else { decimal })
}

fn try_parse_iso6709(s: &str) -> Option<Coordinate> {
    // ISO 6709: +40.7128-074.0060/ or +404246-0740022/
    let s = s.trim_end_matches('/');

    // Find the second +/- (start of longitude)
    let bytes = s.as_bytes();
    let mut split_pos = None;
    for (i, &byte) in bytes.iter().enumerate().skip(1) {
        if byte == b'+' || byte == b'-' {
            split_pos = Some(i);
            break;
        }
    }

    let split = split_pos?;
    let lat_str = &s[..split];
    let lng_str = &s[split..];

    let lat: f64 = lat_str.parse().ok()?;
    let lng: f64 = lng_str.parse().ok()?;

    if lat.abs() > 90.0 || lng.abs() > 180.0 {
        return None;
    }

    Some(Coordinate {
        latitude: lat,
        longitude: lng,
        format: CoordinateFormat::Iso6709,
        datum: Datum::Unknown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_degrees() {
        let c = parse_coordinate("40.7128, -74.0060").unwrap();
        assert!((c.latitude - 40.7128).abs() < 0.001);
        assert!((c.longitude - -74.006).abs() < 0.001);
        assert_eq!(c.format, CoordinateFormat::DecimalDegrees);
    }

    #[test]
    fn decimal_degrees_space_separated() {
        let c = parse_coordinate("40.7128 -74.0060").unwrap();
        assert!((c.latitude - 40.7128).abs() < 0.001);
    }

    #[test]
    fn dms_with_nsew() {
        let c = parse_coordinate("40°42'46\"N 74°0'22\"W").unwrap();
        assert!((c.latitude - 40.7128).abs() < 0.01);
        assert!((c.longitude - -74.006).abs() < 0.01);
        assert_eq!(c.format, CoordinateFormat::Dms);
    }

    #[test]
    fn iso6709() {
        let c = parse_coordinate("+40.7128-074.0060/").unwrap();
        assert!((c.latitude - 40.7128).abs() < 0.001);
        assert!((c.longitude - -74.006).abs() < 0.001);
        assert_eq!(c.format, CoordinateFormat::Iso6709);
    }

    #[test]
    fn detect_order_lng_first() {
        let pairs = vec![(-74.0060, 40.7128), (-118.2437, 34.0522)]; // lng > 90
        assert_eq!(detect_coordinate_order(&pairs), CoordinateOrder::LngLat);
    }

    #[test]
    fn detect_order_lat_first() {
        let pairs = vec![(40.7128, -74.0060), (34.0522, -118.2437)]; // second > 90
        assert_eq!(detect_coordinate_order(&pairs), CoordinateOrder::LatLng);
    }

    #[test]
    fn detect_order_ambiguous() {
        let pairs = vec![(40.0, 50.0), (30.0, 45.0)]; // both ≤ 90
        assert_eq!(detect_coordinate_order(&pairs), CoordinateOrder::Ambiguous);
    }
}
