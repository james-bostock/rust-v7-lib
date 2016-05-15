// Copyright 2015, 2016 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

use std::fs::File;
use std::io;

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

/// Opens the specified file unless its name is "-" in which case the
/// standard input is returned.
pub fn open_file(filename: &str) -> io::Result<Box<io::Read>> {
    if filename == "-" {
        Ok(Box::new(io::stdin()) as Box<io::Read>)
    } else {
        Ok(Box::new(try!(File::open(&filename))) as Box<io::Read>)
    }
}
