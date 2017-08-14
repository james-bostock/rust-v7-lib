// Copyright 2017 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

// An implementation of the tee(1) command in Rust.
// See http://man.cat-v.org/unix-6th/1/tee
use std::env;
use std::fs::File;
use std::io;
use std::io::{Result, Write};

/// A multi-way writer.
struct Tee {
    writers: Vec<Box<Write>>
}

impl Write for Tee {
    /// Writes a buffer to each of the writers, returning how many
    /// bytes were returned by the last write.
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut n: usize = 0;
        for w in &mut self.writers {
            n = (*w).write(buf)?
        }
        Ok(n)
    }

    /// Flushes each writer.
    fn flush(&mut self) -> Result<()> {
        for w in &mut self.writers {
            (*w).flush()?
        }
        Ok(())
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let prog = &args[0];

    if args.len() == 1 {
        eprintln!("usage: {} [name ...]", prog);
    }

    let mut tee: Tee = Tee { writers: Vec::new() };

    tee.writers.push(Box::new(io::stdout()));

    for arg in args.iter().skip(1) {
        match File::create(&arg) {
            Ok(f) => { tee.writers.push(Box::new(f)); },
            Err(e) => { eprintln!("{}: {}: {}", prog, arg, e); }
        }
    }

    io::copy(&mut io::stdin(), &mut tee).expect(prog);
}
