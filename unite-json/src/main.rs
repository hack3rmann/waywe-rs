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

#[derive(Clone)]
pub enum UnitedValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<UnitedValue>),
    Object(HashMap<String, Vec<UnitedValue>>),
}

fn unite(res: &mut UnitedValue, second: Value) {
    todo!()
}

fn format_united_value(val: &UnitedValue, deps: u32) -> String {
    let cur_offset: String = (0..deps).map(|_| "    ").collect();

    match val {
        UnitedValue::Null => cur_offset + "null",
        UnitedValue::Bool(val) => cur_offset + &val.to_string(),
        UnitedValue::Number(val) => cur_offset + &val.to_string(),
        UnitedValue::String(val) => cur_offset + &val,
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
                    if delimiter == " " { "" } else { "\n" }
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

impl std::fmt::Debug for UnitedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::Null => "null".to_string(),
            Self::Bool(val) => val.to_string(),
            Self::Number(val) => val.to_string(),
            Self::String(val) => val.to_owned(),
            Self::Array(val) => format!("{val:?}"),
            Self::Object(val) => {
                let mut res = "{".to_string();

                for (field, possible_values) in val {
                    res.push_str("\"");
                    res.push_str(&field.to_string());
                    res.push_str("\": <");

                    for possible_value in &possible_values[..possible_values.len() - 1] {
                        res.push_str(&format!("{possible_value:?}, "));
                    }
                    res.push_str(&format!("{:?}", possible_values[possible_values.len() - 1]));

                    res.push_str(">");
                }
                res.push_str("}");

                res
            }
        };

        write!(f, "{}", res)
    }
}

fn main() -> io::Result<()> {
    let path = Cli::parse().path;

    let mut res = UnitedValue::Object(Default::default());

    for entry in fs::read_dir(Path::new(&path))? {
        let file_path = entry?.path();

        let fd = fs::File::open(file_path)?;
        let fd = io::BufReader::new(fd);

        let value: Value = serde_json::from_reader(fd)?;

        unite(&mut res, value);
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
    fn display_united_value3() {
        let inner_val = UnitedValue::Array(vec![UnitedValue::Null; 4]);
        let val = UnitedValue::Array(vec![inner_val; 4]);

        println!("{}", format_united_value(&val, 0));
    }
}
