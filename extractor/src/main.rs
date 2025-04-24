use extractor::package;
use extractor::texture::TexExtractData;
use extractor::texture::extract_data;

use std::fs;
use std::io;
use std::io::BufWriter;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about=None)]
/// This program is a useful command line extractor of wallpaper engine inner files
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Extract scene.pkg contents
    Package {
        path: PathBuf,

        out_path: Option<PathBuf>,
    },

    /// Convert .tex files
    Texture {
        paths: Vec<PathBuf>,

        #[arg(long, short)]
        all_mipmaps: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Package { path, out_path } => {
            let fd = fs::File::open(path)?;
            let mut fd = io::BufReader::new(fd);

            let mut reader = package::PackageReader::new(&mut fd)?;

            let out_path = if let Some(out_path) = out_path {
                out_path
            } else {
                "assets".into()
            };

            reader.store_files(&out_path)?;
        }
        Command::Texture { paths, all_mipmaps } => {
            for path in paths {
                let fd = fs::File::open(&path)?;
                let mut fd = io::BufReader::new(fd);

                let data = extract_data(&mut fd)?;

                match data {
                    TexExtractData::Image(images) => {
                        let image = images.into_iter().next().unwrap();

                        if all_mipmaps {
                            for (i, mipmap) in image.mipmaps.iter().enumerate() {
                                let out = fs::File::create(format!(
                                    "{}_{i}.png",
                                    path.file_stem().unwrap().to_str().unwrap()
                                ))?;
                                let out = BufWriter::new(out);

                                let mut encoder =
                                    png::Encoder::new(out, mipmap.width, mipmap.height);
                                encoder.set_color(png::ColorType::Rgba);

                                let mut writer = encoder.write_header()?;
                                writer.write_image_data(mipmap.data.as_bytes())?;
                            }
                        } else {
                            let out = fs::File::create(format!(
                                "{}.png",
                                path.file_stem().unwrap().to_str().unwrap()
                            ))?;
                            let out = BufWriter::new(out);

                            let mut encoder = png::Encoder::new(
                                out,
                                image.mipmaps[0].width,
                                image.mipmaps[0].height,
                            );
                            encoder.set_color(png::ColorType::Rgba);

                            let mut writer = encoder.write_header()?;
                            writer.write_image_data(image.mipmaps[0].data.as_bytes())?;
                        }
                    }
                    TexExtractData::Gif {
                        frames: _frames,
                        frames_meta: _frames_meta,
                    } => {
                        unimplemented!("The file format is gif")
                    }
                    TexExtractData::Video(_video) => {
                        unimplemented!("The file format is video")
                    }
                }
            }
        }
    }

    Ok(())
}
