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
                    // panic!("mismatch types in united_value and value")
                    false
                }
            },
            UnitedValue::Bool(val1) => match other {
                Value::Bool(val2) => val2 == val1,
                _ => {
                    // panic!("mismatch types in united_value and value")
                    false
                }
            },
            UnitedValue::Number(val1) => match other {
                Value::Number(val2) => val2 == val1,
                _ => {
                    // panic!("mismatch types in united_value and value")
                    false
                }
            },
            UnitedValue::String(str1) => match other {
                Value::String(str2) => str1 == str2,
                _ => {
                    // panic!("mismatch types in united_value and value")
                    false
                }
            },
            UnitedValue::Array(arr1) => match other {
                Value::Array(arr2) => arr1.iter().zip(arr2.iter()).all(|(v1, v2)| v1 == v2),
                _ => {
                    // panic!("mismatch types in united_value and value")
                    false
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
                    // panic!("mismatch types in united_value and value")
                    false
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

impl std::fmt::Display for UnitedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format_united_value(self, 0, 0))
    }
}

fn format_united_value(val: &UnitedValue, deps: u32, offset: u32) -> String {
    // This the offset that has to be inserted before print
    let start_tab: String = (0..offset).map(|_| "    ").collect();

    // These two are used for multiline printing
    let current_tab: String = (0..deps).map(|_| "    ").collect();
    let next_tab = current_tab.clone() + "    ";

    match val {
        UnitedValue::Null => start_tab + "null",
        UnitedValue::Bool(val) => format!("{start_tab}{val}"),
        UnitedValue::Number(val) => format!("{start_tab}{val}"),
        UnitedValue::String(val) => format!("{start_tab}\"{val}\""),
        UnitedValue::Array(arr) => {
            let multiline_print =
                arr.len() > 3 || matches!(arr[0], UnitedValue::Array(_) | UnitedValue::Object(_));

            let mut res = start_tab;

            res.push('[');
            if multiline_print {
                res.push('\n');
            }

            let delimiter = if multiline_print { '\n' } else { ' ' };

            for elem in arr {
                if multiline_print {
                    res.push_str(&format_united_value(elem, deps + 1, deps + 1));
                } else {
                    res.push_str(&format_united_value(elem, deps + 1, 0));
                }
                res.push(',');
                res.push(delimiter);
            }

            if multiline_print {
                res.push_str(&format!("{current_tab}"));
            }
            res.push(']');
            res
        }
        UnitedValue::Object(obj) => {
            let mut res = start_tab;

            res.push_str("{\n");

            for key in obj.keys() {
                res.push_str(&next_tab);
                res.push('"');
                res.push_str(key);
                res.push_str("\": ");

                let multiline_print = obj[key].len() > 3
                    || (obj[key].len() > 0
                        && matches!(obj[key][0], UnitedValue::Array(_) | UnitedValue::Object(_)));

                if obj[key].len() > 1 {
                    res.push_str("<");
                }

                if obj[key].len() == 1 {
                    res.push_str(&format!(
                        "{}",
                        &format_united_value(&obj[key][0], deps + 1, 0)
                    ));
                } else {
                    if multiline_print {
                        res.push('\n');
                    }
                    for value in obj[key].iter() {
                        if multiline_print {
                            res.push_str(&format!(
                                "{}",
                                &format_united_value(value, deps + 2, deps + 2)
                            ));
                            res.push_str(",\n");
                        } else {
                            res.push_str(&format!("{}", &format_united_value(value, deps + 2, 0)));
                            res.push_str(", ")
                        }
                    }
                }

                if obj[key].len() > 1 {
                    if multiline_print {
                        res.push_str(&next_tab);
                    }
                    res.push_str(">");
                }
                res.push_str(",\n");
            }

            res.push_str(&format!("\n{current_tab}}}"));
            res
        }
    }
}

fn main() -> io::Result<()> {
    let path = Cli::parse().path;

    let mut res = UnitedValue::Object(Default::default());

    for entry in fs::read_dir(Path::new(&path))? {
        let file_path = entry?.path();

        let fd = fs::File::open(file_path.join("project.json"))?;
        let fd = io::BufReader::new(fd);

        let value: Value = serde_json::from_reader(fd)?;

        unite(&mut res, &value);
    }

    let united_json_fd = fs::File::create("united.json")?;
    let mut united_json_fd = BufWriter::new(united_json_fd);

    united_json_fd.write_all(format!("{}", res).as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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

        println!("{val}");
    }

    #[test]
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

        println!("{val}");
    }

    #[test]
    fn display_united_value_array() {
        let inner_inner_val = UnitedValue::Array(vec![UnitedValue::Null; 3]);
        let inner_val = UnitedValue::Array(vec![inner_inner_val; 3]);
        let val = UnitedValue::Array(vec![inner_val; 3]);

        println!("{val}");
    }

    #[test]
    fn test_unite1() {
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
        println!("unite:\n{first}");
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

        println!("unite2:\n{map}");
    }

    #[test]
    fn test_unite3() {
        let mut map1 = HashMap::new();
        let arr = UnitedValue::Number(serde_json::Number::from_u128(123).unwrap());
        let arr = UnitedValue::Array(vec![arr]);

        map1.insert("b".to_owned(), vec![arr]);
        let mut map = HashMap::new();
        map.insert("a".to_owned(), vec![UnitedValue::Object(map1)]);

        let mut map = UnitedValue::Object(map);

        let arr = Value::Number(serde_json::Number::from_u128(124).unwrap());
        let arr = Value::Array(vec![arr]);

        let mut map1 = serde_json::Map::new();
        map1.insert("b".to_owned(), arr);
        let mut map2 = serde_json::Map::new();
        map2.insert("a".to_owned(), Value::Object(map1));

        let map2 = Value::Object(map2);
        unite(&mut map, &map2);

        println!("unite3:\n{map}");
    }
}
