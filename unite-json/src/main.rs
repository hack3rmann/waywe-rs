// TODO(ArnoDarkrose): rewrite this with HashSet

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use serde_json::Number;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the steam directory with workshop assets for wallpaper engine
    path: String,
}

#[derive(Clone, Debug)]
pub enum UnitedValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<UnitedValue>),
    Object(HashMap<String, Vec<UnitedValue>>),
}

impl PartialEq<Value> for UnitedValue {
    fn eq(&self, other: &Value) -> bool {
        match self {
            UnitedValue::Null => match other {
                Value::Null => true,
                _ => {
                    panic!("mismatch types in united_value and value")
                }
            },
            UnitedValue::Bool(val1) => match other {
                Value::Bool(val2) => val2 == val1,
                _ => {
                    panic!("mismatch types in united_value and value")
                }
            },
            UnitedValue::Number(val1) => match other {
                Value::Number(val2) => val2 == val1,
                _ => {
                    panic!("mismatch types in united_value and value")
                }
            },
            UnitedValue::String(str1) => match other {
                Value::String(str2) => str1 == str2,
                _ => {
                    panic!("mismatch types in united_value and value")
                }
            },
            UnitedValue::Array(arr1) => match other {
                Value::Array(arr2) => arr1.iter().zip(arr2.iter()).all(|(v1, v2)| v1 == v2),
                _ => {
                    panic!("mismatch types in united_value and value")
                }
            },
            UnitedValue::Object(map1) => match other {
                Value::Object(map2) => {
                    for key in map2.keys() {
                        if !map1.contains_key(key) {
                            return false;
                        }

                        if !map1[key].iter().any(|v| v == &map2[key]) {
                            return false;
                        }
                    }

                    true
                }
                _ => {
                    panic!("mismatch types in united_value and value")
                }
            },
        }
    }
}

impl From<Value> for UnitedValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => UnitedValue::Null,
            Value::Bool(val) => UnitedValue::Bool(val),
            Value::Number(val) => UnitedValue::Number(val),
            Value::String(val) => UnitedValue::String(val),
            Value::Array(arr) => UnitedValue::Array(arr.into_iter().map(|v| v.into()).collect()),
            Value::Object(obj) => {
                let mut res = HashMap::new();

                for (key, value) in obj.into_iter() {
                    res.insert(key, vec![value.into()]);
                }
                UnitedValue::Object(res)
            }
        }
    }
}

// returns true if successfully united and false otherwise
fn unite(res: &mut UnitedValue, second: &Value) -> bool {
    if res == second {
        return true;
    }

    match res {
        UnitedValue::Object(united_map) => match second {
            Value::Object(map) => {
                for key in map.keys() {
                    if !united_map.contains_key(key) {
                        united_map.insert(key.to_owned(), vec![map[key].clone().into()]);
                    } else {
                        if !united_map
                            .get_mut(key)
                            .unwrap()
                            .iter_mut()
                            .any(|v| unite(v, &map[key]))
                        {
                            united_map
                                .get_mut(key)
                                .unwrap()
                                .push(map[key].clone().into());
                        }
                    }
                }
            }
            _ => {
                unreachable!()
            }
        },
        _ => return false,
    }

    true
}

fn format_united_value(val: &UnitedValue, deps: u32) -> String {
    let cur_offset: String = (0..deps).map(|_| "    ").collect();

    match val {
        UnitedValue::Null => cur_offset + "null",
        UnitedValue::Bool(val) => cur_offset + &val.to_string(),
        UnitedValue::Number(val) => cur_offset + &val.to_string(),
        UnitedValue::String(val) => format!("\"{}\"", cur_offset + &val),
        UnitedValue::Array(val) => {
            let delimiter = if val.len() > 3 {
                format!(",\n")
            } else {
                ", ".to_string()
            };

            let mut res = if val.len() > 3 {
                format!("{cur_offset}[\n")
            } else {
                format!("{cur_offset}[")
            };

            let cur_deps = if val.len() > 3 { deps + 1 } else { 0 };
            for value in &val[..val.len() - 1] {
                res.push_str(&format!(
                    "{}{delimiter}",
                    format_united_value(value, cur_deps),
                ));
            }
            res.push_str(&format!(
                "{}",
                format_united_value(&val[val.len() - 1], cur_deps)
            ));

            if val.len() > 3 {
                res.push_str(&format!("\n{cur_offset}]"))
            } else {
                res.push_str("]")
            }

            res
        }
        UnitedValue::Object(val) => {
            let mut res = cur_offset.clone() + "{\n";

            let old_offset = cur_offset.clone();
            let cur_offset = cur_offset.clone() + "    ";

            for (field, possible_values) in val {
                res.push_str(&(cur_offset.clone()));
                res.push_str("\"");
                res.push_str(&field.to_string());
                if possible_values.len() > 1 {
                    res.push_str("\": <");
                } else {
                    res.push_str("\": ");
                }

                let delimiter = if matches!(
                    possible_values[0],
                    UnitedValue::Array(_) | UnitedValue::Object(_)
                ) {
                    "\n"
                } else {
                    " "
                };
                let cur_deps = if possible_values.len() == 1
                    && matches!(
                        possible_values[0],
                        UnitedValue::Object(_) | UnitedValue::Array(_)
                    ) {
                    deps + 2
                } else if possible_values.len() > 3
                    || matches!(possible_values[0], UnitedValue::Object(_))
                {
                    deps + 2
                } else {
                    0
                };

                for possible_value in &possible_values[..possible_values.len() - 1] {
                    res.push_str(&format!(
                        "{delimiter}{},",
                        format_united_value(possible_value, cur_deps)
                    ));
                }
                res.push_str(&format!(
                    "{delimiter}{}{}",
                    format_united_value(&possible_values[possible_values.len() - 1], cur_deps),
                    // if delimiter == " " { "" } else { "\n" }
                    delimiter
                ));

                if possible_values.len() <= 1 {
                    res.push_str(&("\n"))
                } else if possible_values.len() > 3
                    || matches!(
                        possible_values[0],
                        UnitedValue::Object(_) | UnitedValue::Array(_)
                    )
                {
                    res.push_str(&(cur_offset.clone() + ">\n"));
                } else {
                    res.push_str(&(">\n"));
                }
            }
            res.push_str(&(old_offset + "}"));

            res
        }
    }
}

// impl std::fmt::Debug for UnitedValue {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let res = match self {
//             Self::Null => "null".to_string(),
//             Self::Bool(val) => val.to_string(),
//             Self::Number(val) => val.to_string(),
//             Self::String(val) => val.to_owned(),
//             Self::Array(val) => format!("{val:?}"),
//             Self::Object(val) => {
//                 let mut res = "{".to_string();

//                 for (field, possible_values) in val {
//                     res.push_str("\"");
//                     res.push_str(&field.to_string());
//                     res.push_str("\": <");

//                     for possible_value in &possible_values[..possible_values.len() - 1] {
//                         res.push_str(&format!("{possible_value:?}, "));
//                     }
//                     res.push_str(&format!("{:?}", possible_values[possible_values.len() - 1]));

//                     res.push_str(">");
//                 }
//                 res.push_str("}");

//                 res
//             }
//         };

//         write!(f, "{}", res)
//     }
// }

fn main() -> io::Result<()> {
    let path = Cli::parse().path;

    let mut res = UnitedValue::Object(Default::default());

    for entry in fs::read_dir(Path::new(&path))? {
        let file_path = entry?.path();

        let fd = fs::File::open(file_path.join("project.json"))?;
        let fd = io::BufReader::new(fd);

        let value: Value = serde_json::from_reader(fd)?;

        unite(&mut res, &value);
        println!("res: {}, value: {value:#?}", format_united_value(&res, 0));
    }

    let united_json_fd = fs::File::create("united.json")?;
    let mut united_json_fd = BufWriter::new(united_json_fd);

    united_json_fd.write_all(format!("{res:?}").as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn debug_united_value() {
        let mut objects = HashMap::new();
        objects.insert(
            "sdf".to_string(),
            vec![UnitedValue::Array(vec![UnitedValue::Bool(false); 3])],
        );
        let val = UnitedValue::Object(objects);

        println!("{val:#?}");
    }

    #[test]
    #[ignore]
    fn display_united_value() {
        let mut objects = HashMap::new();
        objects.insert(
            "sdf".to_string(),
            vec![UnitedValue::Array(vec![UnitedValue::Bool(false); 3])],
        );

        let mut inner_objects = HashMap::new();
        inner_objects.insert(
            "yui".to_string(),
            vec![UnitedValue::String("123".to_owned())],
        );
        objects.insert(
            "jkl".to_string(),
            vec![UnitedValue::Object(inner_objects); 3],
        );
        let val = UnitedValue::Object(objects);

        println!("{}", format_united_value(&val, 0));
    }

    #[test]
    #[ignore]
    fn display_united_value2() {
        let mut objects = HashMap::new();
        objects.insert(
            "sdf".to_string(),
            vec![UnitedValue::Array(vec![UnitedValue::Bool(false); 4])],
        );

        let mut inner_objects = HashMap::new();
        inner_objects.insert(
            "yui".to_string(),
            vec![UnitedValue::String("123".to_owned())],
        );
        objects.insert(
            "jkl".to_string(),
            vec![UnitedValue::Object(inner_objects); 2],
        );
        let val = UnitedValue::Object(objects);

        println!("{}", format_united_value(&val, 0));
    }

    #[test]
    #[ignore]
    fn display_united_value3() {
        let inner_val = UnitedValue::Array(vec![UnitedValue::Null; 4]);
        let val = UnitedValue::Array(vec![inner_val; 4]);

        println!("{}", format_united_value(&val, 0));
    }

    #[test]
    fn test_unite() {
        let mut map = HashMap::new();
        map.insert(
            "a".to_owned(),
            vec![UnitedValue::Number(
                serde_json::Number::from_u128(1).unwrap(),
            )],
        );
        let mut first = UnitedValue::Object(map);

        let mut map = serde_json::Map::new();
        map.insert("a".to_owned(), serde_json::json!(0));
        let second = Value::Object(map);

        unite(&mut first, &second);
        println!("unite:\n{}", format_united_value(&mut first, 0));
    }

    #[test]
    fn test_unite2() {
        let mut map1 = HashMap::new();
        map1.insert(
            "b".to_owned(),
            vec![UnitedValue::Number(
                serde_json::Number::from_u128(123).unwrap(),
            )],
        );
        let mut map = HashMap::new();
        map.insert("a".to_owned(), vec![UnitedValue::Object(map1)]);

        let mut map = UnitedValue::Object(map);

        let mut map1 = serde_json::Map::new();
        map1.insert(
            "b".to_owned(),
            Value::Number(serde_json::Number::from_u128(124).unwrap()),
        );
        let mut map2 = serde_json::Map::new();
        map2.insert("a".to_owned(), Value::Object(map1));

        let map2 = Value::Object(map2);
        unite(&mut map, &map2);

        println!("unite2:\n{}", format_united_value(&map, 0));
    }
}
