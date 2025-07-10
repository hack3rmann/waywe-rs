// TODO(ArnoDarkrose): rewrite this with HashSet
//
// TODO(ArnoDarkrose): maybe make arrays also recurrently unite
// as objects (only viable example is tags, all in all this doesn't sound like a good idea)

use clap::Parser;
use serde_json::{Number, Value};
use std::{
    collections::HashMap,
    fmt, fs,
    io::{self, BufWriter, Write},
    path::Path,
};
use tracing::debug;

const CLI_ABOUT: &str =  "
This binary is used to unite different json files into one big file by keeping only unique fields and enclosing
possible values for each field in <> triagonal braces (braces are omitted if there's only one possible value)
The expected usage is passing the path to the steam directory with workshop assets for wallpaper engine (e.g $GAMES/steamapps/workshop/content/431960)
By default the output is written to the united.json file";

const PATH_ABOUT: &str = "Path to the directory with workshop assets for wallpaper engine.
This has to be a directory with directories each of which containing a project.json in them. Those jsons will be united";

#[derive(Debug, Parser)]
#[command(version, about=CLI_ABOUT, long_about=CLI_ABOUT)]
struct Cli {
    /// Path to the directory with workshop assets for wallpaper engine.
    /// This has to be a directory with directories each of which containing a project.json in them. Those jsons will be united
    #[arg(help = PATH_ABOUT)]
    path: String,

    /// Write to stdout instead of the united.json
    #[arg(long)]
    stdout: bool,
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
        match (self, other) {
            (UnitedValue::Null, Value::Null) => true,
            (UnitedValue::Bool(lhs), Value::Bool(rhs)) => lhs == rhs,
            (UnitedValue::Number(lhs), Value::Number(rhs)) => lhs == rhs,
            (UnitedValue::String(lhs), Value::String(rhs)) => lhs == rhs,
            (UnitedValue::Array(lhs), Value::Array(rhs)) => {
                lhs.iter().zip(rhs.iter()).all(|(v1, v2)| v1 == v2)
            }
            (UnitedValue::Object(lhs), Value::Object(rhs)) => 'object: {
                for key in rhs.keys() {
                    if !lhs.contains_key(key) {
                        break 'object false;
                    }

                    if !lhs[key].iter().any(|v| v == &rhs[key]) {
                        break 'object false;
                    }
                }

                true
            }
            _ => false,
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
            Value::Object(obj) => UnitedValue::Object(
                obj.into_iter()
                    .map(|(key, value)| (key, vec![value.into()]))
                    .collect(),
            ),
        }
    }
}

/// Returns `true` if successfully united and `false` otherwise
fn unite(res: &mut UnitedValue, second: &Value) -> bool {
    if res == second {
        return true;
    }

    match (res, second) {
        (UnitedValue::Object(united_map), Value::Object(map)) => {
            for key in map.keys() {
                if !united_map.contains_key(key) {
                    united_map.insert(key.to_owned(), vec![map[key].clone().into()]);
                } else if !united_map
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
        _ => return false,
    }

    true
}

impl fmt::Display for UnitedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_united_value(self, 0, 0))
    }
}

// NOTE: this skips the values of the `description` field as they are too long and essentialy useless for us
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

            for elem in &arr[..arr.len() - 1] {
                if multiline_print {
                    res.push_str(&format_united_value(elem, deps + 1, deps + 1));
                } else {
                    res.push_str(&format_united_value(elem, deps + 1, 0));
                }

                res.push(',');
                res.push(delimiter);
            }

            if !arr.is_empty() {
                if multiline_print {
                    res.push_str(&format_united_value(
                        &arr[arr.len() - 1],
                        deps + 1,
                        deps + 1,
                    ));
                    res.push_str(",\n");
                } else {
                    res.push_str(&format_united_value(&arr[arr.len() - 1], deps + 1, 0));
                }
            }

            if multiline_print {
                res.push_str(&current_tab.to_string());
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

                if key == "description" {
                    res.push_str(",\n");
                    continue;
                }

                let multiline_print = obj[key].len() > 3
                    || (!obj[key].is_empty()
                        && matches!(obj[key][0], UnitedValue::Array(_) | UnitedValue::Object(_)));

                if obj[key].len() > 1 {
                    res.push('<');
                }

                if obj[key].len() == 1 {
                    res.push_str(&format_united_value(&obj[key][0], deps + 1, 0));
                } else {
                    if multiline_print {
                        res.push('\n');
                    }

                    for value in &obj[key][..obj[key].len() - 1] {
                        if multiline_print {
                            res.push_str(&format_united_value(value, deps + 2, deps + 2));
                            res.push_str(",\n");
                        } else {
                            res.push_str(&format_united_value(value, deps + 2, 0));
                            res.push_str(", ")
                        }
                    }

                    if !obj[key].is_empty() {
                        if multiline_print {
                            res.push_str(&format_united_value(
                                &obj[key][obj[key].len() - 1],
                                deps + 2,
                                deps + 2,
                            ));
                            res.push_str(",\n");
                        } else {
                            res.push_str(&format_united_value(
                                &obj[key][obj[key].len() - 1],
                                deps + 2,
                                0,
                            ));
                        }
                    }
                }

                if obj[key].len() > 1 {
                    if multiline_print {
                        res.push_str(&next_tab);
                    }

                    res.push('>');
                }

                res.push_str(",\n");
            }

            res.push_str(&format!("\n{current_tab}}}"));
            res
        }
    }
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let path = cli.path;

    let mut res = UnitedValue::Object(Default::default());

    for entry in fs::read_dir(Path::new(&path))? {
        let file_path = entry?.path();

        debug!(?file_path, "opening file");
        let fd = fs::File::open(file_path.join("project.json"))?;
        let fd = io::BufReader::new(fd);

        let value: Value = serde_json::from_reader(fd)?;

        unite(&mut res, &value);
    }

    if cli.stdout {
        let fd = io::stdout();
        let mut fd = BufWriter::new(fd);

        fd.write_all(format!("{res}").as_bytes())?;
    } else {
        let fd = fs::File::create("united.json")?;
        let mut fd = BufWriter::new(fd);

        fd.write_all(format!("{res}").as_bytes())?;
    };

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
