// Copyright 2015, 2016 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::fs::File;

// Utility routines

// Taken from stackoverflow:
// http://stackoverflow.com/questions/27588416/how-to-send-output-to-stderr
macro_rules! errln(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

// An input source.
//
// We use an enum so that, when the input is from a file, we can seek on
// it (as needed by the od command).
//
// Taken from stackoverflow:
// http://stackoverflow.com/questions/37223741/how-can-i-take-input-from-either-stdin-or-a-file-if-i-cannot-seek-stdin

/// The `Input` type.
pub enum Input {
    /// Input is from a file
    File(fs::File),
    /// Input is from the standard input
    Stdin(io::Stdin),
}

impl Read for Input {
    /// Pull some bytes from this source into the specified buffer,
    /// returning how many bytes were read.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Input::File(ref mut file) => file.read(buf),
            Input::Stdin(ref mut stdin) => stdin.read(buf),
        }
    }
}

impl Seek for Input {
    /// Seek to an offset, in bytes, in a stream.
    ///
    /// # Errors
    ///
    /// Seeking on the standard input is an error.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match *self {
            Input::File(ref mut file) => file.seek(pos),
            Input::Stdin(_) => {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "not supported by stdin-input",
                ))
            },
        }
    }
}

impl Input {
    /// Create a new Input by opening the specified file or, if the
    /// filename is "-", the standard input.
    pub fn open(filename: &str) -> io::Result<Input> {
        if filename == "-" {
            Ok(Input::Stdin(io::stdin()))
        } else {
            Ok(Input::File(try!(fs::File::open(filename))))
        }
    }
}
