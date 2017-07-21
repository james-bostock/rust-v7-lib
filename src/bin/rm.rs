// Copyright 2016, 2017 James Bostock. See the LICENSE file at the
// top-level directory of this distribution.

// An implementation of the rm(1) command in Rust.
// See http://man.cat-v.org/unix-6th/1/rm
use std::env;
use std::fs;
use std::io;
use std::io::Write;

#[macro_use]
mod util;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut processing_options: bool = true;
    let mut force: bool = false;
    let mut recursive: bool = false;

    if args.len() > 1 {
        for arg in args.iter().skip(1) {
            if arg == "-f" {
                if processing_options {
                    force = true;
                    continue;
                }
            } else if arg == "-r" {
                if processing_options {
                    recursive = true;
                    continue;
                }
            } else {
                processing_options = false;
            }

            match fs::metadata(arg) {
                Ok(md) => {
                    if md.permissions().readonly() {
                        print!("rm: remove readonly file {}? ", arg);
                        io::stdout().flush();
                        let mut resp = String::new();
                        match io::stdin().read_line(&mut resp) {
                            Ok(_) => {
                                match resp.chars().next() {
                                    Some(c) => {
                                        if c != 'y' {
                                            continue;
                                        }
                                    },
                                    None => {}
                                }
                            },
                            Err(e) => {
                                eprintln!("Error reading response: {}", e);
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{}; {}", arg, e);
                    continue;
                }
            }

            match if recursive {
                fs::remove_dir_all(arg)
            } else {
                fs::remove_file(arg)
            } {
                Ok(_) => {},
                Err(e) => { if !force { eprintln!("{}: {}", arg, e); } }
            }
        }
    } else {
        eprintln!("usage: {} [-f][-r] file ...", args[0]);
    }
}
