use crate::SdkError;

#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalValue {
    Null,
    Bool(bool),
    Unsigned {
        value: u64,
        byte_len: u8,
    },
    Negative {
        value: i64,
        byte_len: u8,
    },
    Bytes {
        data: Vec<u8>,
        indefinite: bool,
    },
    Text {
        data: String,
        indefinite: bool,
    },
    Array {
        items: Vec<CanonicalValue>,
        indefinite: bool,
    },
    Map {
        entries: Vec<(String, CanonicalValue)>,
        indefinite: bool,
    },
}

pub fn validate_canonicalization(value: &CanonicalValue) -> Result<(), SdkError> {
    match value {
        CanonicalValue::Null | CanonicalValue::Bool(_) => Ok(()),
        CanonicalValue::Unsigned { value, byte_len } => validate_unsigned(*value, *byte_len),
        CanonicalValue::Negative { value, byte_len } => validate_negative(*value, *byte_len),
        CanonicalValue::Bytes { indefinite, .. } => ensure_definite(*indefinite, "bytes"),
        CanonicalValue::Text { indefinite, .. } => ensure_definite(*indefinite, "text"),
        CanonicalValue::Array { items, indefinite } => {
            ensure_definite(*indefinite, "array")?;
            for item in items {
                validate_canonicalization(item)?;
            }
            Ok(())
        }
        CanonicalValue::Map {
            entries,
            indefinite,
        } => {
            ensure_definite(*indefinite, "map")?;
            validate_map_order(entries)?;
            for (_, value) in entries {
                validate_canonicalization(value)?;
            }
            Ok(())
        }
    }
}

fn ensure_definite(indefinite: bool, label: &str) -> Result<(), SdkError> {
    if indefinite {
        return Err(SdkError::Canonicalization(format!(
            "{label} uses indefinite length encoding"
        )));
    }
    Ok(())
}

fn validate_map_order(entries: &[(String, CanonicalValue)]) -> Result<(), SdkError> {
    let mut last: Option<&String> = None;
    for (key, _) in entries {
        if let Some(previous) = last {
            if key <= previous {
                return Err(SdkError::Canonicalization(
                    "map keys must be strictly lexicographic".to_string(),
                ));
            }
        }
        last = Some(key);
    }
    Ok(())
}

fn validate_unsigned(value: u64, byte_len: u8) -> Result<(), SdkError> {
    let expected = min_bytes_unsigned(value);
    if byte_len != expected {
        return Err(SdkError::Canonicalization(format!(
            "unsigned integer {value} uses {byte_len} bytes (expected {expected})"
        )));
    }
    Ok(())
}

fn validate_negative(value: i64, byte_len: u8) -> Result<(), SdkError> {
    if value >= 0 {
        return Err(SdkError::Canonicalization(
            "negative integer must be less than zero".to_string(),
        ));
    }
    let encoded = (-1_i128 - value as i128) as u64;
    let expected = min_bytes_unsigned(encoded);
    if byte_len != expected {
        return Err(SdkError::Canonicalization(format!(
            "negative integer {value} uses {byte_len} bytes (expected {expected})"
        )));
    }
    Ok(())
}

fn min_bytes_unsigned(value: u64) -> u8 {
    if value <= u8::MAX as u64 {
        1
    } else if value <= u16::MAX as u64 {
        2
    } else if value <= u32::MAX as u64 {
        4
    } else {
        8
    }
}
