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

/// Prompts user for confirmation
fn confirm(msg: &str) -> io::Result<bool> {
    print!("{}: ", msg);
    io::stdout().flush()?;
    let mut resp = String::new();
    io::stdin().read_line(&mut resp)?;
    match resp.chars().next() {
        Some(c) => {
            if c == 'y' {
                Ok(true)
            } else {
                Ok(false)
            }
        },
        None => Ok(false)
    }
}

/// Removes a file or directory
fn rm(name: &str, force: bool, recursive: bool) -> io::Result<()> {
    let md = fs::metadata(name)?;
    let go = if !force && md.permissions().readonly() {
        let mut msg = "rm: remove readonly file ".to_string();
        msg.push_str(&name);
        msg.push_str("?");
        if confirm(&msg)? {
            true
        } else {
            false
        }
    } else {
        true
    };

    if go {
        if recursive {
            fs::remove_dir_all(name)
        } else {
            fs::remove_file(name)
        }
    } else {
        Ok(())
    }
}

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

            match rm(arg, force, recursive) {
                Ok(_) => {},
                Err(e) => eprintln!("{}: {}", arg, e)
            }
        }
    } else {
        eprintln!("usage: {} [-f][-r] file ...", args[0]);
    }
}
