// Copyright 2020, 2021 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

//! A command line parser similar to getopt(3).

use std::fmt;
use std::iter::Iterator;

/// A command line argument (which can be an argument, an option, or an
/// option with an argument).
#[derive(Debug)]
pub enum Arg {
    /// A command line option without an argument.
    Opt(char),
    /// A command line option with an argument.
    OptWithArg(char, String),
    /// A command line argument.
    Arg(String)
}

/// A command line option specification.
#[derive(Clone,Copy,Debug)]
struct OptSpec {
    /// The option character.
    opt: char,
    /// Indicates whether the option expects an argument.
    has_arg: bool
}

impl OptSpec {
    /// Creates a new `OptSpec`.
    fn new(opt: char, has_arg: bool) -> Self {
	OptSpec {opt, has_arg}
    }
}

/// A command line parser similar to getopt(3).
pub struct GetOpt<I>
where
    I: Iterator<Item = String>
{
    /// Indicates whether all options have been parsed.
    opts_done: bool,
    /// The option specifications to use when parsing the command
    /// line.
    opt_specs: Vec<OptSpec>,
    /// The command line arguments.
    args: I,
    /// The letters of the current option argument (including the
    /// leading '-').
    chars: Vec<char>,
    /// The index of the current option in `chars`.
    idx: usize
}

/// Converts a getopt optstring to a vector of `OptSpec`s.
fn parse_optstring(optstring: &str) -> Vec<OptSpec> {
    let mut opt_specs : Vec<OptSpec> = Vec::new();
    let mut last : Option<char> = None;
    for char in optstring.chars() {
	if char == ':' {
	    match last {
		Some(c) => opt_specs.push(OptSpec::new(c, true)),
		None => {
		    panic!("{}: invalid option specification", optstring);
		}
	    }
	} else if char.is_ascii_alphanumeric() {
	    if let Some(c) = last {
		opt_specs.push(OptSpec::new(c, false))
	    }
	    last = Some(char);
	} else {
	    panic!("{}: invalid option specification", optstring);
	}
    }

    if let Some(c) = last {
	opt_specs.push(OptSpec::new(c, false))
    }

    opt_specs
}

impl <I> GetOpt<I>
where
    I: Iterator<Item = String>
{
    /// Creates a new `GetOpt`.
    ///
    /// # Example
    ///
    /// ```
    /// use std::env;
    /// use rust_v7_lib as lib;
    /// let mut args = env::args();
    /// let prog = args.next();
    /// let getopt = lib::GetOpt::new("ab:", args);
    /// for optarg in getopt {
    ///     // Process optarg
    /// }
    /// ```
    pub fn new(optstring: &str, args: I) -> Self {
	let opt_specs = parse_optstring(optstring);
	GetOpt {
	    opt_specs,
	    opts_done: false,
	    args,
	    chars: Vec::new(),
	    idx: 0
	}
    }

    /// Find the option specification matching `opt`.
    fn find_opt_spec(&self, opt: char) -> Option<OptSpec> {
	for opt_spec in &self.opt_specs {
	    if opt_spec.opt == opt {
		return Some(*opt_spec)
	    }
	}

	None
    }

    /// Handle a command line argument.
    fn handle_arg(&mut self, arg: &str) -> Option<Result<Arg, GetOptErr>> {
	if self.opts_done {
	    Some(Ok(Arg::Arg(arg.to_string())))
	} else if arg.starts_with('-') {
	    self.chars = arg.chars().collect();
	    self.idx = 1;
	    if self.chars.len() > 1 {
		if self.chars.len() == 2 && self.chars[1] == '-' {
		    self.opts_done = true;
		    self.idx = 0;
		    match self.args.next() {
			Some(arg) => self.handle_arg(&arg),
			None => None
		    }
		} else {
		    Some(self.handle_option())
		}
	    } else {
		Some(Err(GetOptErr::MissingOpt))
	    }
	} else {
	    self.opts_done = true;
	    Some(Ok(Arg::Arg(arg.to_string())))
	}
    }

    /// Handle a command line option.
    fn handle_option(&mut self) -> Result<Arg, GetOptErr> {
	let opt = self.chars[self.idx];
	self.idx += 1;
	match self.find_opt_spec(opt) {
	    Some(opt_spec) => {
		if opt_spec.has_arg {
		    match self.args.next() {
			Some(arg) => Ok(Arg::OptWithArg(opt, arg)),
			None => Err(GetOptErr::MissingArg(opt))
		    }
		} else {
		    Ok(Arg::Opt(opt))
		}
	    },
	    None => Err(GetOptErr::UnknownOpt(opt))
	}
    }
}

/// The error type for the getopt module.
#[derive(Debug)]
pub enum GetOptErr {
    /// No argument found for a command line option that expects an
    /// argument.
    MissingArg(char),
    /// No option letter found after hyphen.
    MissingOpt,
    /// An unrecognised command line option (i.e. one not present in
    /// the option specification string).
    UnknownOpt(char)
}

impl fmt::Display for GetOptErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	match self {
	    GetOptErr::MissingArg(c) => write!(f, "-{}: expected an argument", c),
	    GetOptErr::MissingOpt => write!(f, "Missing option letter"),
	    GetOptErr::UnknownOpt(c) => write!(f, "-{}: unknown option", c)
	}
    }
}

impl <I> Iterator for GetOpt<I>
where
    I: Iterator<Item = String>
{
    type Item = Result<Arg, GetOptErr>;

    /// Advances the getopt iterator and returns the next command line
    /// argument.
    fn next(&mut self) -> Option<Self::Item> {
	if !self.chars.is_empty() && self.idx > 0 && self.idx < self.chars.len() {
	    Some(self.handle_option())
	} else {
	    match self.args.next() {
		Some(arg) => {
		    self.handle_arg(&arg)
		},
		None => None
	    }
	}
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Assert that a value is a command line argument with the specified
    /// value.
    ///
    /// # Arguments
    ///
    /// * $arg - An Option and Result wrapped Arg instance.
    /// * $val - the value the command line argument is expected to have.
    ///
    /// # example
    ///
    /// The following asserts that the next command line argument is an
    /// argument (as opposed to an option) with the value "ant".
    /// ```
    /// getop_assert_arg!(getopt.next(), "ant")
    /// ```
    macro_rules! getopt_assert_arg {
	($arg:expr, $val:expr) => {
	    match (&$arg, &$val) {
	    	(Some(Ok(Arg::Arg(arg_val))), val_val) => {
		    assert_eq!(arg_val, val_val)
		},
		_ => panic!("Expected argument")
	    }
	}
    }

    /// Assert that a value is a command line option without an option
    /// argument (i.e. a switch).
    ///
    /// # Arguments
    ///
    /// * $arg - An Option ans Result wrapped Arg instance.
    /// * $opt - The expected command line option.
    ///
    /// # example
    ///
    /// The following asserts that the next command line argument is an
    /// option (as opposed to an argument) whose option letter is 'a'.
    /// ```
    /// getop_assert_opt!(getopt.next(), 'a')
    /// ```
    macro_rules! getopt_assert_opt {
	($arg:expr, $opt:expr) => {
	    match (&$arg, &$opt) {
	    	(Some(Ok(Arg::Opt(opt_val))), val_val) => {
		    assert_eq!(opt_val, val_val)
		},
		_ => panic!("Expected option")
	    }
	}
    }


    /// Assert that a value is a command line option with an option
    /// argument.
    ///
    /// # Arguments
    ///
    /// * $arg - An Option ans Result wrapped Arg instance.
    /// * $opt - The expected command line option.
    /// * $val - The expected option value.
    ///
    /// # example
    ///
    /// The following asserts that the next command line argument is an
    /// option (as opposed to an argument) whose option letter is 'a' and
    /// whose option argument is "ant".
    /// ```
    /// getop_assert_opt!(getopt.next(), 'a', "ant")
    /// ```
    macro_rules! getopt_assert_opt_with_arg {
	($arg:expr, $opt:expr, $val:expr) => {
	    match (&$arg, &$opt, &$val) {
	    	(Some(Ok(Arg::OptWithArg(opt_opt, opt_val))), opt, val) => {
		    assert_eq!(opt, opt_opt);
		    assert_eq!(val, opt_val);
		},
		_ => panic!("Expected option")
	    }
	}
    }

    /// Asserts that the last argument has been processed.
    macro_rules! getopt_assert_no_more_args {
	($arg:expr) => {
	    match &$arg {
		Some(a) => panic!("Did not expect argument ({:?})", a),
		None => ()
	    }
	}
    }

    #[test]
    fn test_getopt_arg_only() {
	let args = ["ant"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a", args);
	getopt_assert_arg!(getopt.next(), "ant");
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_opt_only() {
	let args = ["-a"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a", args);
	getopt_assert_opt!(getopt.next(), 'a');
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_opt_with_arg() {
	let args = ["-a", "ant"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a:", args);
	getopt_assert_opt_with_arg!(getopt.next(), 'a', "ant");
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_opt_and_arg() {
	let args = ["-a", "ant"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a", args);
	getopt_assert_opt!(getopt.next(), 'a');
	getopt_assert_arg!(getopt.next(), "ant");
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_missing_arg() {
	let args = ["-a"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a:", args);
	match getopt.next() {
	    Some(arg) => {
		match arg {
		    Err(GetOptErr::MissingArg(opt)) => assert_eq!('a', opt),
		    Err(_) => panic!("Expected MissingArg error"),
		    Ok(_) => panic!("Expected MissingArg error")
		}
	    },
	    None => panic!()
	};
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_unknown_opt() {
	let args = ["-b"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a:", args);
	match getopt.next() {
	    Some(arg) => {
		match arg {
		    Err(GetOptErr::UnknownOpt(opt)) => assert_eq!('b', opt),
		    Err(_) => panic!("Expected UnknownOpt error"),
		    Ok(_) => panic!("Expected UnknownOpt error")
		}
	    },
	    None => panic!()
	};
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_missing_opt() {
	let args = ["-"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a:", args);
	match getopt.next() {
	    Some(arg) => {
		match arg {
		    Err(GetOptErr::MissingOpt) => (),
		    Err(_) => panic!("Expected UnknownOpt error"),
		    Ok(_) => panic!("Expected UnknownOpt error")
		}
	    },
	    None => panic!()
	};
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    #[should_panic]
    fn test_getopt_invalid_optspec1() {
	let args = ["-b"].iter().map(|s| s.to_string());
	let _ = GetOpt::new("!", args);
    }

    #[test]
    #[should_panic]
    fn test_getopt_invalid_optspec2() {
	let args = ["-b"].iter().map(|s| s.to_string());
	let _ = GetOpt::new(":", args);
    }

    #[test]
    fn test_getopt_end_of_opts() {
	let args = ["--", "ant"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a", args);
	getopt_assert_arg!(getopt.next(), "ant");
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_grouped_opts() {
	let args = ["-ab", "-cd"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("abcd", args);
	getopt_assert_opt!(getopt.next(), 'a');
	getopt_assert_opt!(getopt.next(), 'b');
	getopt_assert_opt!(getopt.next(), 'c');
	getopt_assert_opt!(getopt.next(), 'd');
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_grouped_opts_with_args() {
	let args = ["-ab", "ant", "bat"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a:b:", args);
	getopt_assert_opt_with_arg!(getopt.next(), 'a', "ant");
	getopt_assert_opt_with_arg!(getopt.next(), 'b', "bat");
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_grouped_opts_missing_arg() {
	let args = ["-ab"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a:b", args);
	match getopt.next() {
	    Some(arg) => {
		match arg {
		    Err(GetOptErr::MissingArg(opt)) => assert_eq!('a', opt),
		    Err(_) => panic!("Expected MissingArg error"),
		    Ok(_) => panic!("Expected MissingArg error")
		}
	    },
	    None => panic!()
	};
	getopt_assert_opt!(getopt.next(), 'b');
	getopt_assert_no_more_args!(getopt.next());
    }

    #[test]
    fn test_getopt_grouped_opts_with_arg_and_arg() {
	let args = ["-ab", "ant", "bat"].iter().map(|s| s.to_string());
	let mut getopt = GetOpt::new("a:b", args);
	getopt_assert_opt_with_arg!(getopt.next(), 'a', "ant");
	getopt_assert_opt!(getopt.next(), 'b');
	getopt_assert_arg!(getopt.next(), "bat");
	getopt_assert_no_more_args!(getopt.next());
    }
}
