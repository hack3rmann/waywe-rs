use crate::wallpaper::scene::{
    cursor::CursorPlugin, image::ImagePlugin, material::MaterialPlugin, mesh::MeshPlugin,
    transform::TransformPlugin, video::VideoPlugin, wallpaper::Wallpaper,
};
use bitflags::bitflags;
use static_assertions::assert_obj_safe;
use variadics_please::all_tuples;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AddPlugins: u8 {
        const MAIN = 1;
        const RENDER = 2;
    }
}

impl Default for AddPlugins {
    fn default() -> Self {
        Self::MAIN
    }
}

pub trait Plugin {
    fn build(&self, wallpaper: &mut Wallpaper);
}
assert_obj_safe!(Plugin);

pub trait PluginGroup {
    fn add_to_app(self, wallpaper: &mut Wallpaper);
}

impl<P: Plugin> PluginGroup for P {
    fn add_to_app(self, wallpaper: &mut Wallpaper) {
        self.build(wallpaper);
    }
}

#[macro_export]
macro_rules! define_plugin_group {
    {
        $(#[$meta:meta])*
        $vis:vis struct $Group:ident(
            $(
                $Plugin:ident
            ),*
            $(,)?
        );
    } => {
        $(#[$meta])*
        $vis struct $Group
        // NOTE(hack3rmann): with this where clause we achieve better error messages
        where $( $Plugin: PluginGroup ),* ;

        impl PluginGroup for $Group {
            #[allow(unused_variables)]
            fn add_to_app(self, wallpaper: &mut Wallpaper) {
                $(
                    $Plugin.add_to_app(wallpaper);
                )*
            }
        }
    };
}

define_plugin_group! {
    #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub struct DefaultPlugins(
        TransformPlugin,
        ImagePlugin,
        MeshPlugin,
        VideoPlugin,
        MaterialPlugin,
        CursorPlugin,
    );
}

macro_rules! impl_plugin_group_tuple {
    ( $( ($T:ident, $t:ident) ),* ) => {
        impl< $($T),* > PluginGroup for ( $($T,)* )
        where
            $( $T: PluginGroup ),*
        {
            #[allow(unused_variables)]
            fn add_to_app(self, wallpaper: &mut Wallpaper) {
                let ( $($t,)* ) = self;
                $(
                    $t .add_to_app(wallpaper);
                )*
            }
        }
    };
}

all_tuples!(impl_plugin_group_tuple, 0, 15, T, t);
