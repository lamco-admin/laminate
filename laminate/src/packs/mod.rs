//! Domain coercion packs — built-in domain-specific detection, parsing, and conversion.
//!
//! Six packs are always compiled (no feature flags needed):
//!
//! - [`time`] — 14+ date/time formats, ISO 8601 conversion, batch detection, GEDCOM 7.0, HL7 v2
//! - [`currency`] — 30 currency codes, 13 symbols, European/Swiss/Japanese/Indian locale support
//! - [`units`] — Weight, length, temperature (°C↔°F↔K), volume, time, data, UNECE/X12/DOD codes, pack-size notation, SI prefixes, weight qualifiers
//! - [`identifiers`] — IBAN, credit card (Luhn+BIN), ISBN, SSN, EIN, NPI, NHS, VAT, UUID, email, phone
//! - [`geo`] — Decimal degrees, DMS, ISO 6709, lat/lng disambiguation, datum detection
//! - [`medical`] — 18 analyte US↔SI conversions, pharmaceutical notation, HL7 date parsing

pub mod currency;
pub mod geo;
pub mod identifiers;
pub mod medical;
pub mod time;
pub mod units;
