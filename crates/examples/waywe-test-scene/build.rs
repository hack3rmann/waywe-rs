use std::{
    env,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

const USER_AGENT: &str =
    "waywe-test-scene/0.0.15 (https://github.com/hack3rmann/waywe-rs; build-script)";

const ASSETS: &[(&str, &str)] = &[
    (
        "test-image.jpg",
        "https://upload.wikimedia.org/wikipedia/commons/1/11/003_Ringed_kingfisher_flying_with_a_fish_in_Encontro_das_%C3%81guas_State_Park_Photo_by_Giles_Laurent.jpg",
    ),
    (
        "test-image2.jpg",
        "https://upload.wikimedia.org/wikipedia/commons/thumb/a/a9/KAP_Jasa-High_Plateau.jpg/3840px-KAP_Jasa-High_Plateau.jpg",
    ),
    (
        "test-video.mp4",
        "https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4",
    ),
    // (
    //     "test-video2.mp4",
    //     "https://download.samplelib.com/mp4/sample-5s.mp4",
    // ),
    // (
    //     "test-video3.mp4",
    //     "https://download.samplelib.com/mp4/sample-10s.mp4",
    // ),
];

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    let assets_dir = manifest_dir.join("assets");

    if !assets_dir.exists() {
        fs::create_dir_all(&assets_dir).expect("failed to create assets directory");
    }

    for &(filename, url) in ASSETS {
        let dest = assets_dir.join(filename);
        download_if_missing(url, &dest);
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn download_if_missing(url: &str, dest: &Path) {
    if dest.exists() {
        return;
    }

    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .unwrap_or_else(|error| panic!("failed to download {url}: {error}"));

    let status = response.status();
    if !(200..300).contains(&status) {
        panic!("failed to download {url}: HTTP {status}");
    }

    let mut reader = response.into_reader();
    let mut file = File::create(dest)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", dest.display()));
    io::copy(&mut reader, &mut file)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", dest.display()));
}
