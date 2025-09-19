use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use glam::{Vec2, Vec3};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use tracing::warn;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct Scene {
    camera: Camera,
    general: GeneralSettings,
    objects: Vec<Object>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct Object {
    #[serde(deserialize_with = "deserialize_vec3")]
    angles: Vec3,
    id: usize,
    image: Option<String>,
    model: Option<String>,
    name: String,
    #[serde(deserialize_with = "deserialize_vec3")]
    origin: Vec3,
    #[serde(rename = "parallaxDepth")]
    #[serde(deserialize_with = "deserialize_vec2")]
    parallax_depth: Vec2,
    particle: Option<PathBuf>,
    #[serde(rename = "particlesrc")]
    particle_src: Option<()>,
    #[serde(deserialize_with = "deserialize_opt_vec3")]
    #[serde(default)]
    scale: Option<Vec3>,
    #[serde(deserialize_with = "deserialize_opt_vec2")]
    #[serde(default)]
    size: Option<Vec2>,
    visible: Option<bool>,
    #[serde(rename = "colorBlendMode")]
    color_blend_mode: Option<usize>,
    #[serde(rename = "copybackground")]
    copy_background: Option<bool>,
    dependencies: Option<Vec<usize>>,
    #[serde(default)]
    effects: Vec<Effect>,

    // Some fields may not be present above so we capture them here
    #[serde(flatten)]
    #[serde(deserialize_with = "log_deserialize_value")]
    _uncaptured: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct Effect {
    file: PathBuf,
    passes: Vec<Pass>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct Pass {
    // I suppose texture can either be a path or an
    // identificator of some sort like `_rt_imageLayerComposite_12_a`
    textures: Option<Vec<Option<String>>>,
    combos: Option<Combos>,
    #[serde(rename = "constantshadervalues")]
    constant_shader_values: Option<ShaderValues>,

    // Some fields may not be present above so we capture them here
    #[serde(flatten)]
    #[serde(deserialize_with = "log_deserialize_value")]
    _uncaptured: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ShaderValues {
    ui_editor_properties_ray_threshold: Option<f32>,
    #[serde(deserialize_with = "deserialize_opt_vec3")]
    #[serde(default)]
    ui_editor_properties_color_end: Option<Vec3>,
    #[serde(deserialize_with = "deserialize_opt_vec3")]
    #[serde(default)]
    ui_editor_properties_color_start: Option<Vec3>,
    ui_editor_properties_ray_intensity: Option<f32>,
    ui_editor_properties_ray_length: Option<f32>,
    #[serde(deserialize_with = "deserialize_opt_vec2")]
    #[serde(default)]
    ui_editor_properties_blur_scale: Option<Vec2>,
    #[serde(deserialize_with = "deserialize_opt_vec2")]
    #[serde(default)]
    ui_editor_properties_friction: Option<Vec2>,
    ui_editor_properties_speed: Option<f32>,
    ui_editor_properties_strength: Option<f32>,

    // Some fields may not be present above so we capture them here
    #[serde(flatten)]
    #[serde(deserialize_with = "log_deserialize_value")]
    _uncaptured: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub struct Combos {
    vertical: usize,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct GeneralSettings {
    #[serde(deserialize_with = "deserialize_vec3")]
    #[serde(rename = "ambientcolor")]
    ambient_color: Vec3,
    bloom: Property<bool>,
    #[serde(rename = "bloomstrength")]
    bloom_strength: f32,
    #[serde(rename = "camerafade")]
    camera_fade: bool,
    #[serde(rename = "cameraparallax")]
    camera_parallax: Property<bool>,
    #[serde(rename = "cameraparallaxamount")]
    camera_parallax_amount: f32,
    #[serde(rename = "cameraparallaxdelay")]
    camera_parallax_delay: f32,
    #[serde(rename = "cameraparallaxmouseinfluence")]
    camera_parallax_mouse_influence: Property<f32>,
    #[serde(rename = "camerapreview")]
    camera_preview: bool,
    #[serde(rename = "camerashake")]
    camera_shake: Property<bool>,
    #[serde(rename = "camerashakeamplitude")]
    camera_shake_amplitude: f32,
    #[serde(rename = "camerashakeroughness")]
    camera_shake_roughness: f32,
    #[serde(rename = "camerashakespeed")]
    camera_shake_speed: f32,
    #[serde(rename = "clearcolor")]
    #[serde(deserialize_with = "deserialize_vec3")]
    clear_color: Vec3,
    #[serde(rename = "clearenabled")]
    clear_enabled: Option<()>,
    #[serde(rename = "orthogonalprojection")]
    orthogonal_projection: Rectangle,
    #[serde(rename = "skylightcolor")]
    #[serde(deserialize_with = "deserialize_vec3")]
    skylight_color: Vec3,
    zoom: Option<Property<f32>>,
    farz: Option<f32>,
    fov: Option<f32>,
    hdr: Option<bool>,
    nearz: Option<f32>,
    #[serde(rename = "perspectiveoverridefov")]
    perspective_override_fov: Option<f32>,
    #[serde(rename = "gravitydirection")]
    #[serde(deserialize_with = "deserialize_opt_vec3")]
    #[serde(default)]
    gravity_direction: Option<Vec3>,
    #[serde(rename = "gravitystrength")]
    gravity_strength: Option<f32>,
    #[serde(rename = "winddirection")]
    #[serde(deserialize_with = "deserialize_opt_vec3")]
    #[serde(default)]
    wind_direction: Option<Vec3>,
    #[serde(rename = "windenabled")]
    wind_enabled: Option<bool>,
    #[serde(rename = "windstrength")]
    wind_strength: Option<f32>,
    #[serde(rename = "bloomhdrfeather")]
    bloom_hdr_feather: Option<f32>,
    #[serde(rename = "bloomhdrstrength")]
    bloom_hdr_strength: Option<f32>,
    #[serde(rename = "bloomhdriterations")]
    bloom_hdr_iterations: Option<usize>,
    #[serde(rename = "bloomhdrscatter")]
    bloom_hdr_scatter: Option<f32>,
    #[serde(rename = "bloomhdrthreshold")]
    bloom_hdr_threshold: Option<f32>,
    #[serde(rename = "bloomthreshold")]
    bloom_threshold: Option<f32>,
    #[serde(rename = "bloomtint")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_opt_vec3")]
    bloom_tint: Option<Vec3>,

    // Some fields may not be present above so we capture them here
    #[serde(flatten)]
    #[serde(deserialize_with = "log_deserialize_value")]
    _uncaptured: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Property<T> {
    Object(PropertyObject<T>),
    Value(T),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PropertyObject<T> {
    // TODO: check how this script is used. example can be found in 3151551777
    script: Option<String>,
    user: String,
    value: T,
}

impl<T> Default for Property<T>
where
    for<'de> T: Debug + Clone + PartialEq + Default + Deserialize<'de>,
{
    fn default() -> Self {
        Self::Value(T::default())
    }
}

impl<T> Property<T>
where
    for<'de> T: Debug + Clone + PartialEq + Default + Deserialize<'de>,
{
    pub fn value(&self) -> T {
        match self {
            Self::Object(PropertyObject::<T> {
                user: _,
                value,
                script: _,
            }) => value.clone(),
            Self::Value(value) => value.clone(),
        }
    }

    pub fn script(&self) -> Option<&str> {
        match self {
            Self::Object(obj) => obj.script.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rectangle {
    height: usize,
    width: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct Camera {
    #[serde(deserialize_with = "deserialize_vec3")]
    center: Vec3,
    #[serde(deserialize_with = "deserialize_vec3")]
    eye: Vec3,
    #[serde(deserialize_with = "deserialize_vec3")]
    up: Vec3,
}

fn deserialize_vec2<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    str_to_vec2::<D>(&s)
}

fn deserialize_opt_vec2<'de, D>(deserializer: D) -> Result<Option<Vec2>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;

    match opt {
        Some(s) => Some(str_to_vec2::<D>(&s)).transpose(),
        None => Ok(None),
    }
}

fn str_to_vec2<'de, D>(s: &str) -> Result<Vec2, D::Error>
where
    D: Deserializer<'de>,
{
    let nums: Result<Vec<f32>, _> = s.split(" ").map(|s| s.parse()).take(2).collect();
    let nums = nums.map_err(|err| serde::de::Error::custom(err.to_string()))?;

    if nums.len() < 2 {
        return Err(serde::de::Error::invalid_length(
            nums.len(),
            &"expected at least 2 elements",
        ));
    }

    Ok(Vec2::from_slice(nums.as_slice()))
}

fn deserialize_opt_vec3<'de, D>(deserializer: D) -> Result<Option<Vec3>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;

    match opt {
        Some(s) => Some(str_to_vec3::<D>(&s)).transpose(),
        None => Ok(None),
    }
}

fn deserialize_vec3<'de, D>(deserializer: D) -> Result<Vec3, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    str_to_vec3::<D>(&s)
}

fn str_to_vec3<'de, D>(s: &str) -> Result<Vec3, D::Error>
where
    D: Deserializer<'de>,
{
    let nums: Result<Vec<f32>, _> = s.split(" ").map(|s| s.parse()).take(3).collect();
    let nums = nums.map_err(|err| serde::de::Error::custom(err.to_string()))?;

    if nums.len() < 3 {
        return Err(serde::de::Error::invalid_length(
            nums.len(),
            &"expected at least 3 elements",
        ));
    }

    Ok(Vec3::from_slice(nums.as_slice()))
}

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

// TODO: make asserts in tests
#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    // use tracing::Level;
    // use tracing_subscriber::EnvFilter;

    use super::*;

    #[test]
    fn test_deserialize_general() {
        let data = r#"
        	{
        		"ambientcolor" : "0.3 0.3 0.3",
        		"bloom" : false,
        		"bloomstrength" : 2,
        		"bloomthreshold" : 0.64999997615814209,
        		"camerafade" : true,
        		"cameraparallax" : false,
        		"cameraparallaxamount" : 0.5,
        		"cameraparallaxdelay" : 0.10000000149011612,
        		"cameraparallaxmouseinfluence" : 0,
        		"camerapreview" : true,
        		"camerashake" : false,
        		"camerashakeamplitude" : 0.5,
        		"camerashakeroughness" : 1,
        		"camerashakespeed" : 3,
        		"clearcolor" : "0.7 0.7 0.7",
        		"clearenabled" : null,
        		"orthogonalprojection" : 
        		{
        			"height" : 1440,
        			"width" : 2560
        		},
        		"skylightcolor" : "0.3 0.3 0.3"
        	}
        "#;
        let _general: GeneralSettings = serde_json::from_str(data).unwrap();
        // dbg!(general);
    }

    #[test]
    fn test_deserialize_scene_json() {
        // let filter = EnvFilter::default().add_directive(Level::TRACE.into());
        // tracing_subscriber::fmt()
        //     .with_max_level(Level::TRACE)
        //     .with_file(false)
        //     .with_target(false)
        //     .with_env_filter(filter)
        //     .with_writer(std::io::stdout)
        //     .with_line_number(false)
        //     .init();

        let fd = File::open("test-data/monica-scene.json").unwrap();
        let fd = BufReader::new(fd);

        let _scene: Scene = serde_json::from_reader(fd).unwrap();
        // dbg!(_scene);
    }
}
