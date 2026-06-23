//! Domain coercion packs — built-in domain-specific detection, parsing, and conversion.
//!
//! Six packs are always compiled (no feature flags needed):
//!
//! - [`time`] — 18 date/time formats, ISO 8601 conversion, batch detection, GEDCOM 7.0, HL7 v2
//! - [`currency`] — 29 currency codes, 19 symbols, European/Swiss/Japanese/Indian locale support, optional built-in rates
//! - [`units`] — Weight, length, temperature (°C↔°F↔K), volume, time, data, UNECE/X12/DOD codes, pack-size notation, SI prefixes, weight qualifiers
//! - [`identifiers`] — IBAN, credit card (Luhn+BIN), ISBN, SSN, EIN, NPI, NHS, VAT, UUID, email, phone
//! - [`geo`] — Decimal degrees, DMS, DDM, ISO 6709, lat/lng disambiguation, datum detection
//! - [`medical`] — 44 lab-value conversions across 36 analytes, reference ranges, clinical calculators, pharma + HL7/FHIR parsing

pub mod currency;
pub mod geo;
pub mod identifiers;
pub mod medical;
pub mod time;
pub mod units;
