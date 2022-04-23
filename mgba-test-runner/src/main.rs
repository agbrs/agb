#![allow(clippy::all)]

mod runner;
use anyhow::{anyhow, Error};
use image::io::Reader;
use image::GenericImage;
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

enum Timing {
    None,
    WaitFor(i32),
    Difference(i32),
}

const TEST_RUNNER_TAG: u16 = 785;

fn test_file(file_to_run: &str) -> Status {
    let mut finished = Status::Running;
    let debug_reader_mutex = Regex::new(r"(?s)^\[(.*)\] GBA Debug: (.*)$").unwrap();
    let tagged_cycles_reader = Regex::new(r"Cycles: (\d*) Tag: (\d*)").unwrap();

    let mut mgba = runner::MGBA::new(file_to_run).unwrap();
    let video_buffer = mgba.get_video_buffer();
    let mut number_of_cycles = Timing::None;

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
            } else if out.starts_with("Cycles: ") {
                if let Some(captures) = tagged_cycles_reader.captures(out) {
                    let num_cycles: i32 = captures[1].parse().unwrap();
                    let tag: u16 = captures[2].parse().unwrap();

                    if tag == TEST_RUNNER_TAG {
                        number_of_cycles = match number_of_cycles {
                            Timing::WaitFor(n) => Timing::Difference(num_cycles - n),
                            Timing::None => Timing::WaitFor(num_cycles),
                            Timing::Difference(_) => Timing::WaitFor(num_cycles),
                        };
                    }
                }
            } else if out == "[ok]" {
                if let Timing::Difference(cycles) = number_of_cycles {
                    println!(
                        "[ok: {} c ≈ {} s]",
                        cycles,
                        ((cycles as f64 / (16.78 * 1_000_000.0)) * 100.0).round() / 100.0
                    );
                } else {
                    println!("{}", out);
                }
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
    let expected = expected_image.to_rgba8();

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
                let output_file = write_video_buffer(video_buffer);

                return Err(anyhow!(
                    "images do not match, actual output written to {}",
                    output_file
                ));
            }
        }
    }

    Ok(())
}

fn write_video_buffer(video_buffer: &VideoBuffer) -> String {
    let (width, height) = video_buffer.get_size();
    let mut output_image = image::DynamicImage::new_rgba8(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = video_buffer.get_pixel(x, y);
            let pixel_as_rgba = gba_colour_to_rgba(pixel);

            output_image.put_pixel(x, y, pixel_as_rgba.into())
        }
    }

    let output_folder = std::env::temp_dir();
    let output_file = "mgba-test-runner-output.png"; // TODO make this random

    let output_file = output_folder.join(output_file);
    let _ = output_image.save_with_format(&output_file, image::ImageFormat::Png);

    output_file.to_string_lossy().into_owned()
}
