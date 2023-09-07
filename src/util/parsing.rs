use std::collections::BTreeMap;

use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;

/// Attempt to convert a string into the specified type.
pub fn parse_string<T: std::str::FromStr>(input: &str) -> RsrqResult<T> {
    match input.parse::<T>() {
        Ok(val) => Ok(val),
        Err(_) => {
            Err(RsrqError::ParserError(input.to_string()))
        }
    }
}


#[test]
fn test_parse_string() {
    let output_a: u32 = parse_string("1").unwrap();
    assert_eq!(output_a, 1);

    let output_b: u16 = parse_string("1").unwrap();
    assert_eq!(output_b, 1);

    let output_c: bool = parse_string("true").unwrap();
    assert!(output_c);

    let output_d: bool = parse_string("false").unwrap();
    assert!(!output_d);

    let output_e: Result<bool, RsrqError> = parse_string("falze");
    assert!(output_e.is_err());
}

pub fn parse_string_optional<T: std::str::FromStr>(input: &str) -> RsrqResult<Option<T>> {
    if input.is_empty() {
        return Ok(None);
    }
    let val = parse_string(input)?;
    Ok(Some(val))
}

#[test]
fn test_parse_string_optional() {
    let output_a: Option<usize> = parse_string_optional("").unwrap();
    assert!(output_a.is_none());

    let output_b: Option<usize> = parse_string_optional("123").unwrap();
    assert_eq!(output_b.unwrap(), 123);
}


pub fn btree_get<T: std::str::FromStr, K: std::fmt::Display>(map: &BTreeMap<String, String>, key: K) -> RsrqResult<T> {
    let str_value = map.get(&key.to_string()).ok_or(RsrqError::ParserError("Key not found.".to_string()))?;
    parse_string(str_value)
}

pub fn btree_get_opt<T: std::str::FromStr, K: std::fmt::Display>(map: &BTreeMap<String, String>, key: K) -> RsrqResult<Option<T>> {
    let input = map.get(&key.to_string());
    match input {
        Some(str_value) => {
            Ok(parse_string_optional(str_value)?)
        }
        None => Ok(None)
    }
}
