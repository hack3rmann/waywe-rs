use std::{env, path::PathBuf, sync::LazyLock};
use tracing::warn;

/// # The Waywe runtime directory
///
/// Uses `XDG_RUNTIME_DIR` and `WAYLAND_DISPLAY` to construct one.
/// e.g. `/run/user/1000/waywe-wayland-1`
pub static RUNTIME_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    make_runtime_dir(
        env::var("XDG_RUNTIME_DIR").ok(),
        env::var("WAYLAND_DISPLAY").ok(),
    )
});

fn make_runtime_dir(xdg_runtime_dir: Option<String>, wayland_display: Option<String>) -> PathBuf {
    let runtime = xdg_runtime_dir.unwrap_or_else(|| {
        let uid = rustix::process::getuid();
        let fallback = format!("/run/user/{}", uid.as_raw());

        warn!(fallback, "XDG_RUNTIME_DIR env variable is missing");

        fallback
    });

    let mut waywe = wayland_display
        .map(|name_or_path| {
            let Some((_, tail)) = name_or_path.rsplit_once('/') else {
                return name_or_path;
            };
            let Some((head, _)) = tail.rsplit_once(".sock") else {
                return tail.to_owned();
            };
            head.to_owned()
        })
        .unwrap_or_else(|| {
            let fallback = "wayland-0".to_owned();
            warn!(fallback, "WAYLAND_DISPLAY env variable is missing");
            fallback
        });

    waywe.insert_str(0, "waywe-");

    let mut path = PathBuf::from(runtime);
    path.push(waywe);

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_dir_static_initializes() {
        let dir = &*RUNTIME_DIR;
        let name = dir.file_name().unwrap().to_str().unwrap();
        assert!(name.starts_with("waywe-"), "{name}");
    }

    #[test]
    fn some_default() {
        let dir = make_runtime_dir(
            Some("/run/user/1001".to_owned()),
            Some("wayland-42".to_owned()),
        );
        assert_eq!(dir.as_path(), "/run/user/1001/waywe-wayland-42");
    }

    #[test]
    fn strips_absolute_wayland_display_path() {
        let dir = make_runtime_dir(
            Some("/run/user/1001".to_owned()),
            Some("/run/user/1001/wayland-1".to_owned()),
        );
        assert_eq!(dir.as_path(), "/run/user/1001/waywe-wayland-1");
    }

    #[test]
    fn strips_sock_suffix_from_absolute_wayland_display_path() {
        let dir = make_runtime_dir(
            Some("/run/user/1001".to_owned()),
            Some("/run/user/1001/wayland-1.sock".to_owned()),
        );
        assert_eq!(dir.as_path(), "/run/user/1001/waywe-wayland-1");
    }

    #[test]
    fn falls_back_when_xdg_runtime_dir_missing() {
        let uid = rustix::process::getuid().as_raw();
        let dir = make_runtime_dir(None, Some("wayland-1".to_owned()));
        assert_eq!(
            dir,
            PathBuf::from(format!("/run/user/{uid}/waywe-wayland-1"))
        );
    }

    #[test]
    fn falls_back_when_wayland_display_missing() {
        let dir = make_runtime_dir(Some("/run/user/1001".to_owned()), None);
        assert_eq!(dir.as_path(), "/run/user/1001/waywe-wayland-0");
    }

    #[test]
    fn falls_back_when_both_missing() {
        let uid = rustix::process::getuid().as_raw();
        let dir = make_runtime_dir(None, None);
        assert_eq!(
            dir,
            PathBuf::from(format!("/run/user/{uid}/waywe-wayland-0"))
        );
    }
}
