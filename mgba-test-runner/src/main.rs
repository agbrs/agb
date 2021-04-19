#![allow(clippy::all)]

mod runner;
use anyhow::{anyhow, Error};
use io::Write;
use regex::Regex;
use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::rc::Rc;

#[derive(PartialEq, Eq, Debug, Clone)]
enum Status {
    Running,
    Failed,
    Sucess,
}

fn test_file(file_to_run: &str) -> Status {
    let finished = Rc::new(RefCell::new(Status::Running));
    let debug_reader_mutex = Regex::new(r"^\[(.*)\] GBA Debug: (.*)$").unwrap();

    let fin_closure = Rc::clone(&finished);
    runner::set_logger(Box::new(move |message| {
        if let Some(captures) = debug_reader_mutex.captures(message) {
            let log_level = &captures[1];
            let out = &captures[2];

            if out.ends_with("...") {
                print!("{}", out);
                io::stdout().flush().expect("can't flush stdout");
            } else {
                println!("{}", out);
            }

            if log_level == "FATAL" {
                let mut done = fin_closure.borrow_mut();
                *done = Status::Failed;
            }

            if out == "Tests finished successfully" {
                let mut done = fin_closure.borrow_mut();
                *done = Status::Sucess;
            }
        }
    }));

    let mut mgba = runner::MGBA::new(file_to_run);

    loop {
        mgba.advance_frame();
        let done = finished.borrow();
        if *done != Status::Running {
            break;
        }
    }

    runner::clear_logger();

    return (*finished.borrow()).clone();
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
        (((((colour) << 3) & 0xF8) * 0x21) >> 5) as u8,
        (((((colour) >> 2) & 0xF8) * 0x21) >> 5) as u8,
        (((((colour) >> 7) & 0xF8) * 0x21) >> 5) as u8,
        255,
    ]
}
