use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Command,
    str,
};

use image::DynamicImage;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Aseprite {
    pub frames: Vec<Frame>,
    pub meta: Meta,
}

#[derive(Deserialize)]
pub struct Meta {
    pub app: String,
    pub version: String,
    pub image: String,
    pub format: String,
    pub size: Size,
    pub scale: String,
    #[serde(rename = "frameTags")]
    pub frame_tags: Vec<FrameTag>,
}

#[derive(Deserialize)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Forward,
    Backward,
    Pingpong,
}

#[derive(Deserialize, Clone)]
pub struct FrameTag {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub direction: Direction,
}

#[derive(Deserialize, Clone)]
pub struct Frame {
    pub frame: Frame2,
    pub trimmed: bool,
}

#[derive(Deserialize, Clone)]
pub struct Frame2 {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub fn generate_from_file(filename: &str) -> (Aseprite, DynamicImage) {
    let out_dir = std::env::var("OUT_DIR").expect("Expected OUT_DIR");

    let output_filename = Path::new(&out_dir).join(&*filename);
    let image_output = output_filename.with_extension("png");
    let json_output = output_filename.with_extension("json");

    let command = Command::new("aseprite")
        .args([
            &PathBuf::from("-b"),
            &PathBuf::from(filename),
            &"--sheet".into(),
            &image_output,
            &"--format".into(),
            &"json-array".into(),
            &"--data".into(),
            &json_output,
            &"--list-tags".into(),
        ])
        .output()
        .expect("Could not run aseprite");
    assert!(
        command.status.success(),
        "Aseprite did not complete successfully : {}",
        str::from_utf8(&*command.stdout).unwrap_or("Output contains invalid string")
    );

    let json: Aseprite = serde_json::from_reader(
        File::open(&json_output).expect("The json output from aseprite could not be openned"),
    )
    .expect("The output from aseprite could not be decoded");

    (
        json,
        image::open(image_output).expect("Image should be readable"),
    )
}
