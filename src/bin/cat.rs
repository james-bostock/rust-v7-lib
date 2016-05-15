// Copyright 2015, 2016 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

// An implementation of the cat(1) command in Rust.
// See http://man.cat-v.org/unix-6th/1/cat
use std::env;
use std::io;
use std::io::Write;

#[path = "../lib/util.rs"]
#[macro_use]
mod util;

fn cat(filename: &str) -> io::Result<u64> {
    let mut reader = try!(util::open_file(&filename));
    io::copy(&mut reader, &mut io::stdout())
}

fn main() {
    let mut args: Vec<_> = env::args().collect();

    if args.len() == 1 {
        args.push("-".to_string());
    }

    for arg in args.iter().skip(1) {
        match cat(&arg) {
            Ok(_) => {}
            Err(e) => {
                errln!("{}: {}", arg, e);
            }
        };
    }
}
