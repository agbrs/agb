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

#[derive(Deserialize)]

pub struct FrameTag {
    pub name: String,
    pub from: u32,
    pub to: u32,
    pub direction: Direction,
}

#[derive(Deserialize)]
pub struct Frame {
    pub frame: Frame2,
    pub trimmed: bool,
}

#[derive(Deserialize)]
pub struct Frame2 {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
