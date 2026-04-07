//! Iteration 323: FlexValue wrapping a root-level array [1,2,3].
//! Test each("") behavior — empty path on root array.

use laminate::FlexValue;

#[test]
fn root_array_each_empty_path() {
    let fv = FlexValue::from_json(r#"[1, 2, 3]"#).unwrap();
    let items = fv.each("");
    println!("each('') on root array: {} items", items.len());
    for (i, item) in items.iter().enumerate() {
        println!("  [{}] = {:?}", i, item);
    }
    // Empty path should return root value; if root is array, each should iterate it
    assert_eq!(
        items.len(),
        3,
        "root array each('') should yield 3 elements"
    );
}

#[test]
fn root_array_at_index() {
    let fv = FlexValue::from_json(r#"[10, 20, 30]"#).unwrap();
    let val: i64 = fv.extract("[0]").unwrap();
    assert_eq!(val, 10);
    let val: i64 = fv.extract("[2]").unwrap();
    assert_eq!(val, 30);
}

#[test]
fn root_array_len() {
    let fv = FlexValue::from_json(r#"[1, 2, 3, 4]"#).unwrap();
    let len = fv.len();
    println!("root array len: {:?}", len);
    assert_eq!(len, Some(4));
}
