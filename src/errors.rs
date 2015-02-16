// Copyright 2015 Mikhail Zabaluev <mikhail.zabaluev@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error::{Error, FromError};
use std::fmt;
use std::io;

#[derive(Copy)]
pub struct DecodeError {
    desc: &'static str
}

impl DecodeError {
    pub fn new(description: &'static str) -> DecodeError {
        DecodeError { desc: description }
    }
}

impl Error for DecodeError {
    fn description(&self) -> &str { self.desc }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.desc)
    }
}

impl fmt::Debug for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DecodeError(\"{}\")", self.desc)
    }
}

impl FromError<DecodeError> for io::Error {
    fn from_error(err: DecodeError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidInput, err.desc, None)
    }
}
