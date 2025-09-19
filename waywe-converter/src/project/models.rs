use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Deserializer, de::Unexpected};
use serde_json::Value;
use tracing::warn;

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Project {
    #[serde(rename = "contentrating")]
    content_rating: String,
    description: String,
    file: PathBuf,
    preview: PathBuf,
    tags: Vec<String>,
    title: String,
    r#type: ProjectType,
    visibility: Visibility,
    // TODO: i remember this can also be a string and maybe something else
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_workshopid")]
    workshopid: Option<usize>,
    general: GeneralProperties,

    // Some fields may not be captured above so we capture them here
    #[serde(deserialize_with = "log_deserialize_value")]
    #[serde(flatten)]
    _uncaptured: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct GeneralProperties {
    properties: HashMap<String, Property>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Property {
    order: usize,
    text: String,
    r#type: PropertyType,
    value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    #[default]
    Color,
}

#[derive(Debug, Clone, Hash, Default, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
}

#[derive(Debug, Clone, Hash, Default, PartialEq, Deserialize)]
pub enum ProjectType {
    #[serde(alias = "scene")]
    Scene,
    #[default]
    #[serde(alias = "video")]
    Video,
    #[serde(alias = "web")]
    Web,
}

// TODO: consider moving this somewhere as this function is also used in another modesl vile
fn log_deserialize_value<'de, D>(deserializer: D) -> Result<HashMap<String, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let fields = HashMap::<String, Value>::deserialize(deserializer)?;

    if !fields.is_empty() {
        warn!(
            ?fields,
            "encountered unknown fields during deserialization, "
        );
    }

    Ok(fields)
}

fn deserialize_workshopid<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(val) = Option::<Value>::deserialize(deserializer)? else {
        return Ok(None);
    };

    match val {
        Value::Number(num) => Ok(num.as_u64().ok_or(serde::de::Error::custom(&format!(
            "failed to parse workshopid, {num}"
        )))? as usize)
        .map(|v| v.into()),
        Value::String(str) => Ok(str
            .parse::<usize>()
            .map_err(|err| serde::de::Error::custom(err.to_string()))?)
        .map(|v| v.into()),
        Value::Bool(val) => Err(serde::de::Error::invalid_value(
            Unexpected::Bool(val),
            &"expected string or number",
        )),
        Value::Array(_) => Err(serde::de::Error::invalid_value(
            Unexpected::Seq,
            &"expected string or number",
        )),
        Value::Object(obj) => Err(serde::de::Error::invalid_value(
            Unexpected::Other(&format!("encountered: {obj:?}")),
            &"expected string or number",
        )),
        Value::Null => Err(serde::de::Error::invalid_value(
            Unexpected::Other("got null"),
            &"expected string or number",
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    // use tracing::Level;
    // use tracing_subscriber::EnvFilter;

    use crate::project::models::Project;

    #[test]
    fn test_deserialize_project_json() {
        // let filter = EnvFilter::default().add_directive(Level::TRACE.into());
        // tracing_subscriber::fmt()
        //     .with_max_level(Level::TRACE)
        //     .with_file(false)
        //     .with_target(false)
        //     .with_env_filter(filter)
        //     .with_writer(std::io::stdout)
        //     .with_line_number(false)
        //     .init();

        let fd = File::open("test-data/monica-project.json").unwrap();
        let fd = BufReader::new(fd);

        let proj: Project = serde_json::from_reader(fd).unwrap();
        dbg!(proj);
    }
}
