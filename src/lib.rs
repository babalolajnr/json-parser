use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, peek, recognize},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

pub fn parse_string(input: &str) -> IResult<&str, String> {
    let (input, string) = delimited(
        char('"'),
        escaped(is_not("\\\""), '\\', char('"')),
        char('"'),
    )(input)?;
    Ok((input, string.to_owned()))
}

pub fn parse_number(input: &str) -> IResult<&str, f64> {
    let integer_parser = map_res(digit1, |s: &str| s.parse::<f64>());
    let integer_parser_2 = map_res(digit1, |s: &str| s.parse::<f64>());

    let fractional_parser = map_res(digit1, |s: &str| s.parse::<f64>())
        .map(|fractional| fractional / 10f64.powi(fractional.to_string().len() as i32));

    let mut number_parser = alt((
        recognize(tuple((integer_parser, char('.'), fractional_parser))),
        recognize(integer_parser_2),
    ));

    number_parser(input).map(|(remaining, number)| (remaining, number.parse().unwrap()))
}

pub fn parse_boolean(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

pub fn parse_null(input: &str) -> IResult<&str, ()> {
    map(tag("null"), |_| ())(input)
}

pub fn parse_value(input: &str) -> IResult<&str, JsonValue> {
    preceded(
        multispace0,
        alt((
            parse_object,
            parse_array,
            map(parse_string, JsonValue::String),
            map(parse_number, JsonValue::Number),
            map(parse_boolean, JsonValue::Boolean),
            map(parse_null, |_| JsonValue::Null),
        )),
    )(input)
}

pub fn parse_object(input: &str) -> IResult<&str, JsonValue> {
    let parse_pair = tuple((parse_string, preceded(multispace0, char(':')), parse_value));
    let parse_object = delimited(
        preceded(multispace0, char('{')),
        separated_list0(preceded(multispace0, char(',')), parse_pair),
        preceded(multispace0, char('}')),
    );
    map(parse_object, |pairs| {
        let mut object = Vec::new();
        for (key, _, value) in pairs {
            object.push((key, value));
        }
        JsonValue::Object(object)
    })(input)
}

pub fn parse_array(input: &str) -> IResult<&str, JsonValue> {
    let parse_array = delimited(
        preceded(multispace0, char('[')),
        separated_list0(preceded(multispace0, char(',')), parse_value),
        preceded(multispace0, char(']')),
    );
    map(parse_array, JsonValue::Array)(input)
}

pub fn parse_json(input: &str) -> IResult<&str, JsonValue> {
    preceded(multispace0, parse_value)(input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_string_test() {
        assert_eq!(
            super::parse_string(r#""Hello, World!""#),
            Ok(("", "Hello, World!".to_owned()))
        );
    }

    #[test]
    fn parse_decimal_number_test() {
        assert_eq!(super::parse_number("123.456"), Ok(("", 123.456)));
    }

    #[test]
    fn parse_integer_number_test() {
        assert_eq!(super::parse_number("123"), Ok(("", 123.0)));
    }
}

// #[test]
// fn parse_json_test() {
//     let input = r#"{
//         "name": "John Doe",
//         "age": 42,
//         "isStudent": true,
//         "grades": [90, 85, 95],
//         "address": {
//             "street": "123 Main St",
//             "city": "Anytown",
//             "state": "CA",
//             "zip": "12345"
//         },
//         "phoneNumbers": [
//             {"type": "home", "number": "555-1234"},
//             {"type": "work", "number": "555-5678"}
//         ]
//     }"#;

//     assert_eq!(
//         parse_json(input),
//         Ok((
//             "",
//             JsonValue::Object(vec![
//                 ("name".to_owned(), JsonValue::String("John Doe".to_owned())),
//                 ("age".to_owned(), JsonValue::Number(42.0)),
//                 ("isStudent".to_owned(), JsonValue::Boolean(true)),
//                 (
//                     "grades".to_owned(),
//                     JsonValue::Array(vec![
//                         JsonValue::Number(90.0),
//                         JsonValue::Number(85.0),
//                         JsonValue::Number(95.0),
//                     ])
//                 ),
//                 (
//                     "address".to_owned(),
//                     JsonValue::Object(vec![
//                         (
//                             "street".to_owned(),
//                             JsonValue::String("123 Main St".to_owned())
//                         ),
//                         ("city".to_owned(), JsonValue::String("Anytown".to_owned())),
//                         ("state".to_owned(), JsonValue::String("CA".to_owned())),
//                         ("zip".to_owned(), JsonValue::String("12345".to_owned())),
//                     ])
//                 ),
//                 (
//                     "phoneNumbers".to_owned(),
//                     JsonValue::Array(vec![
//                         JsonValue::Object(vec![
//                             ("type".to_owned(), JsonValue::String("home".to_owned())),
//                             (
//                                 "number".to_owned(),
//                                 JsonValue::String("555-1234".to_owned())
//                             ),
//                         ]),
//                         JsonValue::Object(vec![
//                             ("type".to_owned(), JsonValue::String("work".to_owned())),
//                             (
//                                 "number".to_owned(),
//                                 JsonValue::String("555-5678".to_owned())
//                             ),
//                         ]),
//                     ])
//                 ),
//             ])
//         ))
//     );
// }
