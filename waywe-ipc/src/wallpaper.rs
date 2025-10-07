use bincode::{Decode, Encode};

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Encode, Decode)]
pub enum WallpaperType {
    #[default]
    Video,
    Image,
    Scene,
}
