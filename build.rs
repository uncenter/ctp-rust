use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use serde::Deserialize;

const PALETTE_URL: &str =
    "https://raw.githubusercontent.com/catppuccin/palette/v0.2.0/palette-porcelain.json";
const PALETTE_PATH: &str = ".cache/palette.json";
const CODEGEN_PATH: &str = "./src/palette.rs";

#[derive(Debug, Deserialize)]
struct Color {
    hex: String,
    rgb: [u8; 3],
    hsl: [f32; 3],
}

type Palette = HashMap<String, HashMap<String, Color>>;

fn is_dark(name: &str) -> bool {
    name != "latte"
}

fn is_accent(name: &str) -> bool {
    [
        "rosewater",
        "flamingo",
        "pink",
        "mauve",
        "red",
        "maroon",
        "peach",
        "yellow",
        "green",
        "teal",
        "sky",
        "sapphire",
        "blue",
        "lavender",
    ]
    .contains(&name)
}

const HEADER: &str = "// DO NOT MODIFY THIS FILE!
// The contents of this file are generated by build.rs automatically.
// Any changes you make here WILL be overwritten.
#![allow(clippy::unreadable_literal)]
use crate::{Palette, Flavor, Color};
";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={}", PALETTE_PATH);

    let palette_path = Path::new(PALETTE_PATH);

    if !palette_path.exists() {
        println!("cargo:warning=no palette json in cache, fetching...");
        download_palette(palette_path)?;
    }

    let palette: Palette = serde_json::from_reader(BufReader::new(File::open(palette_path)?))?;

    let codegen_file = BufWriter::new(File::create(CODEGEN_PATH)?);
    let mut code_writer = BufWriter::new(codegen_file);

    write!(&mut code_writer, "{HEADER}")?;

    let mut flavors_map = phf_codegen::Map::new();
    for (flavor_name, flavor) in palette.into_iter() {
        let mut colors_map = phf_codegen::Map::new();
        for (color_name, color) in flavor.into_iter() {
            let color_entry = make_color_entry(&color_name, color);
            colors_map.entry(color_name, &color_entry);
        }
        let flavor_entry = make_flavor_entry(&flavor_name, colors_map);
        flavors_map.entry(flavor_name, &flavor_entry);
    }
    writeln!(
        &mut code_writer,
        "pub static PALETTE: Palette = Palette {{ flavors: {} }};",
        flavors_map.build()
    )?;

    Ok(())
}

fn download_palette(path: &Path) -> Result<(), Box<dyn Error>> {
    let contents = ureq::get(PALETTE_URL).call()?.into_string()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

fn make_color_entry(name: &str, color: Color) -> String {
    format!(
        r#"Color {{
            name: "{}",
            is_accent: {},
            hex: "{}",
            rgb: &[{}, {}, {}],
            hsl: &[{}.0, {}, {}],
        }}"#,
        name,
        is_accent(name),
        color.hex,
        color.rgb[0],
        color.rgb[1],
        color.rgb[2],
        color.hsl[0],
        color.hsl[1],
        color.hsl[2]
    )
}

fn make_flavor_entry(name: &str, colors_map: phf_codegen::Map<String>) -> String {
    format!(
        r#"Flavor {{
            name: "{}",
            dark: {},
            colors: {},
        }}"#,
        name,
        is_dark(name),
        colors_map.build()
    )
}
