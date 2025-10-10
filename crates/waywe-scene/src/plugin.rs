//! Plugin system for extending wallpaper functionality.
//!
//! Plugins are the primary way to add functionality to a wallpaper scene.
//! They can add systems, resources, and configure schedules in both the
//! main and render worlds.
//!
//! # Built-in Plugins
//!
//! The scene system provides several built-in plugins:
//!
//! - [`TransformPlugin`]: Handle entity positioning and hierarchy
//! - [`ImagePlugin`]: Load and display images
//! - [`MeshPlugin`]: Render geometric shapes
//! - [`VideoPlugin`]: Play video content
//! - [`MaterialPlugin`]: Define surface appearance
//! - [`CursorPlugin`]: Handle cursor interaction
//!
//! These are combined in [`DefaultPlugins`] for convenience.
//!
//! # Custom Plugins
//!
//! To create a custom plugin, implement the [`Plugin`] trait:
//!
//! ```rust
//! use waywe_scene::{plugin::Plugin, wallpaper::Wallpaper};
//!
//! pub struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn build(&self, wallpaper: &mut Wallpaper) {
//!         // Add your systems and resources here
//!     }
//! }
//! ```
//!
//! Plugins can then be added to a wallpaper:
//!
//! ```rust,ignore
//! wallpaper.add_plugins(MyPlugin);
//! ```

use crate::{
    asset_server::AssetServerPlugin, clear_screen::ClearScreenPlugin, cursor::CursorPlugin,
    image::ImagePlugin, material::MaterialPlugin, mesh::MeshPlugin, sprite::SpritePlugin,
    transform::TransformPlugin, video::VideoPlugin, wallpaper::Wallpaper,
};
use bitflags::bitflags;
use static_assertions::assert_obj_safe;
use variadics_please::all_tuples;

bitflags! {
    /// Controls which worlds a plugin should be added to.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AddPlugins: u8 {
        /// Add to the main world.
        const MAIN = 1;
        /// Add to the render world.
        const RENDER = 2;
    }
}

impl Default for AddPlugins {
    fn default() -> Self {
        Self::MAIN
    }
}

/// A plugin that can add functionality to a wallpaper.
///
/// Plugins are the primary way to extend wallpaper functionality.
/// They can add systems, resources, and configure schedules.
pub trait Plugin {
    /// Build the plugin by adding systems and resources to the wallpaper.
    fn build(&self, wallpaper: &mut Wallpaper);
}
assert_obj_safe!(Plugin);

/// A group of plugins that can be added together.
///
/// Plugin groups allow you to combine multiple plugins into a single unit.
pub trait PluginGroup {
    /// Add all plugins in this group to the wallpaper.
    fn add_to_app(self, wallpaper: &mut Wallpaper);
}

impl<P: Plugin> PluginGroup for P {
    fn add_to_app(self, wallpaper: &mut Wallpaper) {
        self.build(wallpaper);
    }
}

/// Macro to define a plugin group from a list of plugins.
///
/// This macro creates a new struct that implements [`PluginGroup`] and
/// adds all the specified plugins when added to a wallpaper.
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
    /// The default set of plugins for a wallpaper.
    ///
    /// This plugin group includes all the essential plugins for creating
    /// dynamic wallpapers:
    /// - [`TransformPlugin`]: Entity positioning
    /// - [`ImagePlugin`]: Image loading and rendering
    /// - [`MeshPlugin`]: Geometric shapes
    /// - [`VideoPlugin`]: Video playback
    /// - [`MaterialPlugin`]: Surface appearance
    /// - [`CursorPlugin`]: Cursor interaction
    #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub struct DefaultPlugins(
        TransformPlugin,
        CursorPlugin,
        AssetServerPlugin,
        MaterialPlugin,
        ImagePlugin,
        VideoPlugin,
        MeshPlugin,
        ClearScreenPlugin,
        SpritePlugin,
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
