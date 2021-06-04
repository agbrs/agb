#![allow(clippy::all)]

mod runner;
use anyhow::{anyhow, Error};
use image::io::Reader;
use io::Write;
use regex::Regex;
use runner::VideoBuffer;
use std::io;
use std::path::Path;

#[derive(PartialEq, Eq, Debug, Clone)]
enum Status {
    Running,
    Failed,
    Sucess,
}

fn test_file(file_to_run: &str) -> Status {
    let mut finished = Status::Running;
    let debug_reader_mutex = Regex::new(r"^\[(.*)\] GBA Debug: (.*)$").unwrap();

    let mut mgba = runner::MGBA::new(file_to_run);
    let video_buffer = mgba.get_video_buffer();

    mgba.set_logger(|message| {
        if let Some(captures) = debug_reader_mutex.captures(message) {
            let log_level = &captures[1];
            let out = &captures[2];

            if out.starts_with("image:") {
                let image_path = out.strip_prefix("image:").unwrap();
                match check_image_match(image_path, &video_buffer) {
                    Err(e) => {
                        println!("[failed]");
                        println!("{}", e);
                        finished = Status::Failed;
                    }
                    Ok(_) => {}
                }
            } else if out.ends_with("...") {
                print!("{}", out);
                io::stdout().flush().expect("can't flush stdout");
            } else {
                println!("{}", out);
            }

            if log_level == "FATAL" {
                finished = Status::Failed;
            }

            if out == "Tests finished successfully" {
                finished = Status::Sucess;
            }
        }
    });

    loop {
        mgba.advance_frame();
        if finished != Status::Running {
            break;
        }
    }

    return finished;
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    let file_to_run = args.get(1).expect("you should provide file to run");

    if !Path::new(file_to_run).exists() {
        return Err(anyhow!("File to run should exist!"));
    }

    let output = test_file(file_to_run);

    match output {
        Status::Failed => Err(anyhow!("Tests failed!")),
        Status::Sucess => Ok(()),
        _ => {
            unreachable!("very bad thing happened");
        }
    }
}

fn gba_colour_to_rgba(colour: u32) -> [u8; 4] {
    [
        ((colour >> 0) & 0xFF) as u8,
        ((colour >> 8) & 0xFF) as u8,
        ((colour >> 16) & 0xFF) as u8,
        255,
    ]
}

fn rgba_to_gba_to_rgba(c: [u8; 4]) -> [u8; 4] {
    let mut n = c.clone();
    n.iter_mut()
        .for_each(|a| *a = ((((*a as u32 >> 3) << 3) * 0x21) >> 5) as u8);
    n
}

fn check_image_match(image_path: &str, video_buffer: &VideoBuffer) -> Result<(), Error> {
    let expected_image = Reader::open(image_path)?.decode()?;
    let expected = expected_image
        .as_rgba8()
        .ok_or(anyhow!("cannot convert to rgba8"))?;

    let (buf_dim_x, buf_dim_y) = video_buffer.get_size();
    let (exp_dim_x, exp_dim_y) = expected.dimensions();
    if (buf_dim_x != exp_dim_x) || (buf_dim_y != exp_dim_y) {
        return Err(anyhow!("image sizes do not match"));
    }
    for y in 0..buf_dim_y {
        for x in 0..buf_dim_x {
            let video_pixel = video_buffer.get_pixel(x, y);
            let image_pixel = expected.get_pixel(x, y);
            let video_pixel = gba_colour_to_rgba(video_pixel);
            let image_pixel = rgba_to_gba_to_rgba(image_pixel.0);
            if image_pixel != video_pixel {
                return Err(anyhow!("images do not match"));
            }
        }
    }

    Ok(())
}
