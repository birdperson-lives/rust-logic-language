#![feature(box_patterns)]

// use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;

mod grammar;
mod error;
mod ast;
mod types;
mod state;

use ast::*;
use error::FileLocation;
use error::SourceInfo;
use state::Bindings;


fn main() {
    let base_context = FileLocation::new("<prelude>", 0, 0);
    let stdout = io::stdout();
    for filename in env::args().skip(1) {
        match SourceInfo::new(&filename, &base_context) {
            Ok(source_info) => {
                let mut f = File::open(filename.clone()).unwrap();
                let mut file_text: String = "".to_string();

                if let Err(_) = f.read_to_string(&mut file_text) {
                    panic!("failed to read file {}", filename);
                }

                let mut locals = LocalBindings::new();
                let mut globals = Bindings::new();
                // println!("parsing");
                match grammar::parse_Program(&mut locals, &mut globals, &source_info, &stdout, &file_text) {
                    Ok(_)  => println!("ran"),
                    Err(err) => println!("{}", err),
                }
                // println!("parsed");
            },
            Err(rlang_err) => {
                let mut handle = stdout.lock();
                rlang_err.to_console_noexcerpt(&mut handle);
            }
        }
    }
}