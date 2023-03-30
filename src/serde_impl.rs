use serde::de;

/// Deserialize an instance of type `T` from bytes of JSON text.
///
/// Note that this needs to take a mutable reference to the bytes because
/// it performs some modification in place before deserializing.
pub fn from_slice<'a, T>(v: &'a mut [u8]) -> serde_json::Result<T>
where
    T: de::Deserialize<'a>,
{
    crate::translate_slice(v);
    serde_json::from_slice(v)
}

#[test]
fn test_deserialize() {
    let mut json = br#"[Infinity, -Infinity, NaN]"#.to_vec();
    let rv: serde_json::Value = from_slice(&mut json[..]).unwrap();
    assert_eq!(
        rv,
        serde_json::Value::Array(vec![
            serde_json::Value::Number(serde_json::Number::from(0)),
            serde_json::Value::Number(serde_json::Number::from_f64(-0.0).unwrap()),
            serde_json::Value::Number(serde_json::Number::from(0)),
        ])
    );
}
