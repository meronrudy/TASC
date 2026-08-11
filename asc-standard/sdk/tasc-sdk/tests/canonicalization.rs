use tasc_sdk::canonicalization::{validate_canonicalization, CanonicalValue};

#[test]
fn map_keys_must_be_lexicographic() {
    let value = CanonicalValue::Map {
        indefinite: false,
        entries: vec![
            ("b".to_string(), CanonicalValue::Null),
            ("a".to_string(), CanonicalValue::Null),
        ],
    };

    let err = validate_canonicalization(&value).expect_err("should fail");
    assert!(err.to_string().contains("lexicographic"));
}

#[test]
fn map_keys_accept_sorted() {
    let value = CanonicalValue::Map {
        indefinite: false,
        entries: vec![
            ("a".to_string(), CanonicalValue::Null),
            ("b".to_string(), CanonicalValue::Null),
        ],
    };

    validate_canonicalization(&value).expect("should pass");
}

#[test]
fn rejects_indefinite_lengths() {
    let value = CanonicalValue::Bytes {
        data: vec![1, 2, 3],
        indefinite: true,
    };

    let err = validate_canonicalization(&value).expect_err("should fail");
    assert!(err.to_string().contains("indefinite"));
}

#[test]
fn enforces_shortest_numeric_encoding() {
    let value = CanonicalValue::Unsigned {
        value: 256,
        byte_len: 1,
    };

    let err = validate_canonicalization(&value).expect_err("should fail");
    assert!(err.to_string().contains("expected"));

    let ok = CanonicalValue::Unsigned {
        value: 255,
        byte_len: 1,
    };

    validate_canonicalization(&ok).expect("should pass");
}
