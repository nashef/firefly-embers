use std::collections::BTreeMap;
use std::str::FromStr;

use firefly_client::models::Uri;
use firefly_client::rendering::*;
use secp256k1::PublicKey;

#[test]
fn test_serialize_bool() {
    let result = true.into_value();
    let expected = Value::Bool(true);
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_i8() {
    let result = 1i8.into_value();
    let expected = Value::Int(1);
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_i16() {
    let result = 1i16.into_value();
    let expected = Value::Int(1);
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_i32() {
    let result = 1i32.into_value();
    let expected = Value::Int(1);
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_i64() {
    let result = 1i64.into_value();
    let expected = Value::Int(1);
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_str() {
    let result = "str".into_value();
    let expected = Value::String("str".to_owned());
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_none() {
    let result = None::<String>.into_value();
    let expected = Value::Nil;
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_some() {
    let result = Some("str").into_value();
    let expected = Value::String("str".into());
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_unit() {
    let result = ().into_value();
    let expected = Value::Tuple(Default::default());
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_newtype_struct() {
    #[derive(IntoValue)]
    struct NewType(String);

    let result = NewType("str".to_owned()).into_value();
    let expected = Value::String("str".into());
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_seq() {
    let result = vec!["foo", "bar"].into_value();

    let expected = Value::List(vec![
        Value::String("foo".into()),
        Value::String("bar".into()),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_tuple() {
    let result = ("foo", "bar").into_value();

    let expected = Value::Tuple(vec![
        Value::String("foo".into()),
        Value::String("bar".into()),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_tuple_struct() {
    #[derive(IntoValue)]
    struct TupleStruct(String, String);

    let result = TupleStruct("foo".into(), "bar".into()).into_value();

    let expected = Value::Tuple(vec![
        Value::String("foo".into()),
        Value::String("bar".into()),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_map() {
    let result = std::iter::once(("foo", "bar"))
        .collect::<BTreeMap<_, _>>()
        .into_value();

    let expected =
        Value::Map(std::iter::once(("foo".to_owned(), Value::String("bar".to_owned()))).collect());
    assert_eq!(result, expected);
}

#[test]
fn test_serialize_struct() {
    #[derive(IntoValue)]
    struct Struct {
        name: String,
        second_name: String,
    }

    let result = Struct {
        name: "foo".into(),
        second_name: "bar".into(),
    }
    .into_value();

    let expected = Value::Map(
        [
            ("name".to_owned(), Value::String("foo".into())),
            ("second_name".to_owned(), Value::String("bar".into())),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(result, expected);
}

#[test]
fn test_render_nil() {
    assert_eq!(Value::Nil.to_string(), "Nil");
}

#[test]
fn test_render_bool() {
    assert_eq!(Value::Bool(true).to_string(), "true");
}

#[test]
fn test_render_number() {
    assert_eq!(Value::Int(13).to_string(), "13");
}

#[test]
fn test_render_string() {
    assert_eq!(Value::String("foo".into()).to_string(), "\"foo\"");
}

#[test]
fn test_render_uri() {
    assert_eq!(Value::Uri("foo".into()).to_string(), "`foo`");
}

#[test]
fn test_render_tuple() {
    assert_eq!(
        Value::Tuple(vec![Value::Nil, Value::String("foo".into())]).to_string(),
        "(Nil, \"foo\")"
    );
}

#[test]
fn test_render_list() {
    assert_eq!(
        Value::List(vec![Value::Nil, Value::String("foo".into())]).to_string(),
        "[Nil, \"foo\"]"
    );
}

#[test]
fn test_render_map() {
    assert_eq!(
        Value::Map(
            [
                ("val1".to_owned(), Value::Nil),
                ("val2".to_owned(), Value::String("foo".into())),
            ]
            .into_iter()
            .collect()
        )
        .to_string(),
        "{\"val1\": Nil, \"val2\": \"foo\"}"
    );
}

#[test]
fn test_serialize_str_is_escaped() {
    let cases = [("foo\\", "\"foo\\\\\""), ("\"foo\"", "\"\\\"foo\\\"\"")];
    for (value, expected) in cases {
        let result = value.into_value();
        assert_eq!(result.to_string(), expected);
    }
}

#[test]
fn test_build_uri_from_public_key() {
    let public_key = PublicKey::from_str("04f4b4417f930e6fab5765ac0defcf9fce169982acfd046e7c27f9b14c0804014623c0439e5c8035e9607599a549303b5b6b90cd9685e6965278bddca65dac7510").unwrap();
    let uri = Uri::from(public_key);
    assert_eq!(
        uri.as_ref(),
        "rho:id:1qw5ehmq1x49dey4eadr1h4ncm361w3536asho7dr38iyookwcsp6i"
    );
}

#[test]
fn test_build_uri_from_string() {
    let uri =
        Uri::try_from("rho:id:1qw5ehmq1x49dey4eadr1h4ncm361w3536asho7dr38iyookwcsp6i".to_owned())
            .unwrap();
    assert_eq!(
        uri.as_ref(),
        "rho:id:1qw5ehmq1x49dey4eadr1h4ncm361w3536asho7dr38iyookwcsp6i"
    );
}
