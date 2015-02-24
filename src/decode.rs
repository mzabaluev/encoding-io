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
    fn decode<'a, 'b>(&'a mut self, input: &'b [u8]) -> Result<Decoded<'a, 'b>>;
    fn output(&self) -> &str;
    fn consume(&mut self);
    fn reset(&mut self);
}

pub type Result<T> = StdResult<T, DecodeError>;

#[derive(Debug)]
pub enum Decoded<'a, 'b> {
    Some(usize, &'a str),
    InPlace(&'b str)
}

impl<'a, 'b> Decoded<'a, 'b> {

    #[inline]
    pub fn some(input_len: usize, output: &'a str) -> Result<Decoded<'a, 'b>> {
        Ok(Decoded::Some(input_len, output))
    }

    #[inline]
    pub fn in_place(part: &'b str) -> Result<Decoded<'a, 'b>> {
        Ok(Decoded::InPlace(part))
    }
}
