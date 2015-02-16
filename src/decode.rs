// Copyright 2015 Mikhail Zabaluev <mikhail.zabaluev@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use errors::DecodeError;

use std::result::Result as StdResult;

pub trait Decode {
    fn decode<'a>(&'a mut self, input: &'a [u8]) -> Result<Decoded<'a>>;
    fn reset(&mut self) -> Result<()>;
}

pub type Result<T> = StdResult<T, DecodeError>;

pub struct Decoded<'a> {
    consumed: usize,
    output: &'a str
}

impl<'a> Decoded<'a> {

    #[inline]
    pub fn ok(input_len: usize, output: &'a str) -> Result<Decoded<'a>> {
        Ok(Decoded { consumed: input_len, output: output })
    }

    #[inline]
    pub fn input_len(&self) -> usize { self.consumed }

    #[inline]
    pub fn output(&self) -> &'a str { self.output }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.consumed == 0 && self.output.is_empty()
    }
}
