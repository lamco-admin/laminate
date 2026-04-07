//! Iteration 247: Nested path + pack coercion propagation
//!
//! Tests that pack coercion propagates through at() navigation.
//! If you set PackCoercion::Currency on the root and navigate to
//! "items[0].price", the currency pack should still fire.
//!
//! Adversarial: what about extract("items[0].price") — does it work
//! through the dotted path? What about each_iter followed by extract?

use laminate::value::PackCoercion;
use laminate::CoercionLevel;
use laminate::FlexValue;

#[test]
fn pack_coercion_through_dotted_path() {
    let data = serde_json::json!({
        "order": {
            "items": [
                {"name": "Widget", "price": "$12.99"},
                {"name": "Gadget", "price": "$24.50"}
            ],
            "total": "$37.49"
        }
    });

    let val = FlexValue::from(data)
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    // Extract through nested dotted path
    let price0: f64 = val.extract("order.items[0].price").unwrap();
    assert!(
        (price0 - 12.99).abs() < 0.01,
        "should extract nested currency"
    );

    let price1: f64 = val.extract("order.items[1].price").unwrap();
    assert!((price1 - 24.50).abs() < 0.01);

    let total: f64 = val.extract("order.total").unwrap();
    assert!((total - 37.49).abs() < 0.01);
}

#[test]
fn pack_coercion_through_at_then_extract() {
    let data = serde_json::json!({
        "order": {
            "total": "$37.49"
        }
    });

    let val = FlexValue::from(data)
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    // Navigate with at(), then extract
    let order = val.at("order").unwrap();
    let total: f64 = order.extract("total").unwrap();
    assert!(
        (total - 37.49).abs() < 0.01,
        "pack coercion should survive at() navigation"
    );
}

#[test]
fn pack_coercion_through_each_iter() {
    let data = serde_json::json!({
        "prices": ["$10.00", "$20.00", "$30.00"]
    });

    let val = FlexValue::from(data)
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Currency);

    // each_iter should propagate pack_coercion to each element
    let prices: Vec<f64> = val
        .each_iter("prices")
        .map(|item| item.extract_root::<f64>().unwrap())
        .collect();

    println!("Prices: {:?}", prices);
    assert_eq!(prices.len(), 3);
    assert!((prices[0] - 10.0).abs() < 0.01);
    assert!((prices[1] - 20.0).abs() < 0.01);
    assert!((prices[2] - 30.0).abs() < 0.01);
}

#[test]
fn units_pack_through_nested_path() {
    let data = serde_json::json!({
        "shipment": {
            "weight": "15.5 kg",
            "dimensions": {
                "length": "100 cm",
                "width": "50 cm"
            }
        }
    });

    let val = FlexValue::from(data)
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::Units);

    let weight: f64 = val.extract("shipment.weight").unwrap();
    assert!((weight - 15.5).abs() < 0.01);

    let length: f64 = val.extract("shipment.dimensions.length").unwrap();
    assert!((length - 100.0).abs() < 0.01);

    let width: f64 = val.extract("shipment.dimensions.width").unwrap();
    assert!((width - 50.0).abs() < 0.01);
}

#[test]
fn mixed_packs_through_nested_paths() {
    let data = serde_json::json!({
        "invoice": {
            "total": "$1,234.56",
            "weight": "10 kg",
            "items": 5
        }
    });

    let val = FlexValue::from(data)
        .with_coercion(CoercionLevel::BestEffort)
        .with_pack_coercion(PackCoercion::All);

    let total: f64 = val.extract("invoice.total").unwrap();
    assert!((total - 1234.56).abs() < 0.01);

    let weight: f64 = val.extract("invoice.weight").unwrap();
    assert!((weight - 10.0).abs() < 0.01);

    // Non-pack value should just use normal coercion
    let items: i64 = val.extract("invoice.items").unwrap();
    assert_eq!(items, 5);
}
