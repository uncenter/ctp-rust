use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use serde::Deserialize;

const PALETTE_URL: &str =
    "https://raw.githubusercontent.com/catppuccin/palette/v1.0.3/palette.json";
const PALETTE_PATH: &str = ".cache/palette-v1.0.3.json";
const CODEGEN_PATH: &str = "./src/generated_palette.rs";

#[derive(Debug, Deserialize)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Deserialize)]
struct Hsl {
    h: f64,
    s: f64,
    l: f64,
}

#[derive(Debug, Deserialize)]
struct Color {
    hex: String,
    rgb: Rgb,
    hsl: Hsl,
    accent: bool,
}

#[derive(Debug, Deserialize)]
struct Flavor {
    name: String,
    dark: bool,
    colors: HashMap<String, Color>,
}

type Palette = HashMap<String, Flavor>;

const HEADER: &str = "// DO NOT MODIFY THIS FILE!
// The contents of this file are generated by build.rs automatically.
// Any changes you make here WILL be overwritten.
#![allow(clippy::unreadable_literal)]
use crate::{Palette, Flavor, Color, FlavorColors, Rgb, Hsl};
";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={PALETTE_PATH}");

    let palette_path = Path::new(PALETTE_PATH);

    if !palette_path.exists() {
        println!("cargo:warning=no palette json in cache, fetching...");
        download_palette(palette_path)?;
    }

    let palette: Palette = serde_json::from_reader(BufReader::new(File::open(palette_path)?))?;

    let codegen_file = BufWriter::new(File::create(CODEGEN_PATH)?);
    let mut code_writer = BufWriter::new(codegen_file);

    write!(&mut code_writer, "{HEADER}")?;

    make_palette_const(&mut code_writer, &palette)?;

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

fn make_palette_const<W: Write>(w: &mut W, palette: &Palette) -> Result<(), Box<dyn Error>> {
    writeln!(
        w,
        "/// The Catppuccin palette. This constant will generally be your entrypoint
/// into using the crate.
pub const PALETTE: Palette = Palette {{"
    )?;
    for (flavor_name, flavor) in palette {
        write!(w, "    {flavor_name}: ")?;
        make_flavor_entry(w, flavor)?;
    }
    writeln!(w, "}};")?;
    Ok(())
}

fn make_flavor_entry<W: Write>(w: &mut W, flavor: &Flavor) -> Result<(), Box<dyn Error>> {
    writeln!(
        w,
        "Flavor {{
        name: {:?},
        dark: {:?},
        colors: FlavorColors {{",
        flavor.name, flavor.dark
    )?;
    for (color_name, color) in &flavor.colors {
        write!(w, "            {color_name}: ")?;
        make_color_entry(w, color, color_name)?;
    }
    writeln!(w, "        }},\n    }},")?;
    Ok(())
}

fn make_color_entry<W: Write>(w: &mut W, color: &Color, name: &str) -> Result<(), Box<dyn Error>> {
    writeln!(
        w,
        r#"Color {{
                name: {:?},
                accent: {:?},
                hex: {:?},
                rgb: Rgb {{ r: {:?}, g: {:?}, b: {:?} }},
                hsl: Hsl {{ h: {:?}, s: {:?}, l: {:?} }},
            }},"#,
        name,
        color.accent,
        color.hex,
        color.rgb.r,
        color.rgb.g,
        color.rgb.b,
        color.hsl.h,
        color.hsl.s,
        color.hsl.l,
    )?;
    Ok(())
}