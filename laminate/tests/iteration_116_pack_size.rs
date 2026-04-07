/// Iteration 116: Pack-size notation parser
///
/// Parses supply chain pack-size expressions into structured PackSize.
use laminate::packs::units::parse_pack_notation;

#[test]
fn multiplier_count() {
    let ps = parse_pack_notation("1x100-count").unwrap();
    assert_eq!(ps.total_units, 100);
    assert_eq!(ps.packs, Some(1));

    let ps = parse_pack_notation("3x100-count").unwrap();
    assert_eq!(ps.total_units, 300);
    assert_eq!(ps.packs, Some(3));
}

#[test]
fn multiplier_with_unit() {
    let ps = parse_pack_notation("6x500ml").unwrap();
    assert_eq!(ps.total_units, 3000);
    assert_eq!(ps.packs, Some(6));
    assert!(ps.per_pack.is_some());
    assert_eq!(ps.per_pack.unwrap().unit, "mL");
}

#[test]
fn case_of() {
    let ps = parse_pack_notation("case of 12").unwrap();
    assert_eq!(ps.total_units, 12);

    let ps = parse_pack_notation("box of 24").unwrap();
    assert_eq!(ps.total_units, 24);
}

#[test]
fn count_suffix() {
    let ps = parse_pack_notation("48-ct").unwrap();
    assert_eq!(ps.total_units, 48);

    let ps = parse_pack_notation("48ct").unwrap();
    assert_eq!(ps.total_units, 48);
}

#[test]
fn named_quantities() {
    assert_eq!(parse_pack_notation("each").unwrap().total_units, 1);
    assert_eq!(parse_pack_notation("EA").unwrap().total_units, 1);
    assert_eq!(parse_pack_notation("dozen").unwrap().total_units, 12);
    assert_eq!(parse_pack_notation("gross").unwrap().total_units, 144);
}

#[test]
fn pack_notation() {
    let ps = parse_pack_notation("6-pack").unwrap();
    assert_eq!(ps.total_units, 6);

    let ps = parse_pack_notation("pk/12").unwrap();
    assert_eq!(ps.total_units, 12);
}

#[test]
fn invalid_returns_none() {
    assert!(parse_pack_notation("").is_none());
    assert!(parse_pack_notation("hello world").is_none());
}
