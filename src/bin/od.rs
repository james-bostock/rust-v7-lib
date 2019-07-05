// Copyright 2016-2018 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

// An implementation of the od(1) command in Rust.
// See http://man.cat-v.org/unix-6th/1/od
use std::env;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Stdout;
use std::io::Write;
use std::num::ParseIntError;

mod util;

/// Writes a chunk of output data as octal byte values.
fn write_oct_bytes(out: &mut BufWriter<Stdout>, data: &[u8])
                   -> io::Result<usize> {
    for byte in data {
        write!(out, " {:03o}", byte)?;
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Writes a chunk of output data as octal (16 bit) word values. Words are
/// assumed to be little endian.
fn write_oct_words(out: &mut BufWriter<Stdout>, data: &[u8])
                   -> io::Result<usize> {
    for word in data.chunks(2) {
        write!(out, "  {:06o}", u16::from(word[1]) << 8 | u16::from(word[0]))?;
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Writes a chunk of output data as decimal (16 bit) word values. Words are
/// assumed to be little endian.
fn write_dec_words(out: &mut BufWriter<Stdout>, data: &[u8])
                   -> io::Result<usize> {
    for word in data.chunks(2) {
        write!(out, "  {:06}", u16::from(word[1]) << 8 | u16::from(word[0]))?;
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Writes a chunk of output data as hexadecimal (16 bit) word values. Words
/// are assumed to be little endian.
fn write_hex_words(out: &mut BufWriter<Stdout>, data: &[u8])
                   -> io::Result<usize> {
    for word in data.chunks(2) {
        write!(out, "  {:06x}", u16::from(word[1]) << 8 | u16::from(word[0]))?;
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Writes a chunk of data as ASCII, reverting to octal byte values for
/// non-printable characters. Standard escape sequences are supported.
fn write_ascii_chars(out: &mut BufWriter<Stdout>, data: &[u8])
                     -> io::Result<usize> {
    for byte in data {
        match *byte {
            7u8 => write!(out, " \\g")?,
            8u8 => write!(out, " \\b")?,
            9u8 => write!(out, " \\t")?,
            10u8 => write!(out, " \\n")?,
            11u8 => write!(out, " \\v")?,
            12u8 => write!(out, " \\f")?,
            13u8 => write!(out, " \\r")?,
            _ => if *byte < 32u8 || *byte > 126u8 {
                write!(out, " {:03o}", *byte)?
            } else {
                write!(out, "   {}", *byte as char)?
            }
        }
    }
    writeln!(out)?;
    Ok(data.len())
}

const CHUNK_SIZE: usize = 16;

// The offset string is of the form [+]offset[.][b]
// +100 => 0o100
// +100. => 100
// +100b => 0o100 * 512
// +100.b => 100 * 512
fn parse_offset(offstr: &str) -> Result<u64, ParseIntError> {
    let mut char_indices = offstr.char_indices().rev();
    let mut mult = 1;
    let (s, r) = match char_indices.next() {
        Some((x, 'b')) => {
            mult = 512;
            match char_indices.next() {
                Some((y, '.')) => (&offstr[0..y], 10),
                Some((_, _)) => (&offstr[0..x], 8),
                None => (&offstr[0..0], 8)
            }
        },
        Some((x, '.')) => (&offstr[0..x], 10),
        Some((_, _)) => (&offstr[..], 8),
        None => (&offstr[0..0], 8)
    };

    match u64::from_str_radix(s, r) {
        Ok(n) => Ok(n * mult),
        Err(e) => Err(e)
    }
}

#[test]
fn test_parse_offset() {
    match parse_offset("100") {
        Ok(off) => assert!(off == 0o100),
        Err(_) => assert!(false)
    }

    match parse_offset("100.") {
        Ok(off) => assert!(off == 100),
        Err(_) => assert!(false)
    }

    match parse_offset("100b") {
        Ok(off) => assert!(off == 0o100 * 512),
        Err(_) => assert!(false)
    }

    match parse_offset("100.b") {
        Ok(off) => assert!(off == 100 * 512),
        Err(_) => assert!(false)
    }
}

/// Dumps the data read from the named input source to the standard output.
fn od(filename: &str, offset: u64,
      fmts: &[fn(&mut BufWriter<Stdout>,&[u8]) -> io::Result<usize>])
      -> io::Result<u64> {
    let mut reader = BufReader::new(util::Input::open(filename)?);
    let mut writer = BufWriter::new(io::stdout());
    let mut offset = offset;

    if offset > 0 {
        reader.seek(SeekFrom::Start(offset))?;
    }

    let mut chunk = [0; CHUNK_SIZE];
    loop {
        let n = reader.read(&mut chunk)?;
        if n > 0 {
            let mut first = true;
            for fmt in fmts.iter() {
                if first {
                    write!(writer, "{:07o}", offset)?;
                    first = false;
                } else {
                    write!(writer, "       ")?;
                }
                fmt(&mut writer, &chunk)?;
                offset += chunk.len() as u64;
            }
        }

        if n < CHUNK_SIZE {
            break
        }
    }
    writeln!(writer, "{:07o}", offset)?;
    Ok(offset)
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let mut idx = 0;
    let mut offset : u64 = 0;
    let mut offstr = "0";
    let mut fmts: Vec<fn(&mut BufWriter<Stdout>, &[u8]) -> io::Result<usize>> = Vec::new();

    if args.len() > idx && args[idx].starts_with('-') {
        for opt in args[idx].chars().skip(1) {
            match opt {
                'b' => fmts.push(write_oct_bytes),
                'c' => fmts.push(write_ascii_chars),
                'd' => fmts.push(write_dec_words),
                'h' => fmts.push(write_hex_words),
                'o' => fmts.push(write_oct_words),
                _ => println!("-{}: unrecognised option", opt),
            }
        }
        idx += 1;
    }

    // If no output formats have been specified, default to octal words.
    if fmts.is_empty() {
        fmts.push(write_oct_words);
    }

    let mut filename = String::from("-");
    if args.len() > idx {
        if args[idx].starts_with('+') {
            offstr = &args[idx][..];
        } else {
            filename = String::from(&args[idx][..]);
            idx += 1;
        }
    }

    if args.len() > idx {
        offstr = &args[idx][..];
    }

    match parse_offset(&offstr) {
        Ok(off) => offset = off,
        Err(e) => println!("{}: {}", offstr, e)
    }

    match od(&filename, offset, &fmts) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1)
        }
    }
}
