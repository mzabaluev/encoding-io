// Copyright 2015 Mikhail Zabaluev <mikhail.zabaluev@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use decode;
use decode::{Decode, Decoded};
use errors::DecodeError;

use std::cmp::min;
use std::slice::bytes;
use std::str;

pub struct Utf8 {
    // Length of the sequence accumulated in `stitch`
    stitch_len: u32,
    // Room for an UTF-8 sequence plus a byte for a speculative copy
    stitch: [u8; 5]
}

impl Utf8 {
    pub fn new() -> Utf8 {
        Utf8 { stitch: [0; 5], stitch_len: 0 }
    }
}

macro_rules! seq_error {
    () => {
        return Err(DecodeError::new("stream did not contain valid UTF-8"));
    }
}

macro_rules! match_seq {
    ($s:expr, $off:expr, $ensure_len:ident, $p1:pat) => {
        $ensure_len! {
            ($s, $off + 1) {
                match $s[$off] {
                    $p1 => Ok($off + 1),
                    _   => seq_error!()
                }
            }
        }
    };
    ($s:expr, $off:expr, $ensure_len:ident, $p1:pat, $p2:pat) => {
        $ensure_len! {
            ($s, $off + 1) {
                match $s[$off] {
                    $p1 => match_seq!($s, $off + 1, $ensure_len, $p2),
                    _   => seq_error!()
                }
            }
        }
    };
    ($s:expr, $off:expr, $ensure_len:ident, $p1:pat, $p2:pat, $p3:pat) => {
        $ensure_len! {
            ($s, $off + 1) {
                match $s[$off] {
                    $p1 => match_seq!($s, $off + 1, $ensure_len, $p2, $p3),
                    _   => seq_error!()
                }
            }
        }
    };
}

macro_rules! validate_next_impl {
    ($s:expr, $ensure_len:ident) => {
        match $s[0] {
            0x00 ... 0x7F => Ok(1),
            0xC2 ... 0xDF =>
                match_seq!($s, 1, $ensure_len,
                           0x80 ... 0xBF),
            0xE0 =>
                match_seq!($s, 1, $ensure_len,
                           0xA0 ... 0xBF, 0x80 ... 0xBF),
            0xE1 ... 0xEC | 0xEE ... 0xEF =>
                match_seq!($s, 1, $ensure_len,
                           0x80 ... 0xBF, 0x80 ... 0xBF),
            0xED => 
                match_seq!($s, 1, $ensure_len,
                           0x80 ... 0x9F, 0x80 ... 0xBF),
            0xF0 =>
                match_seq!($s, 1, $ensure_len,
                           0x90 ... 0xBF, 0x80 ... 0xBF, 0x80 ... 0xBF),
            0xF1 ... 0xF3 =>
                match_seq!($s, 1, $ensure_len,
                           0x80 ... 0xBF, 0x80 ... 0xBF, 0x80 ... 0xBF),
            0xF4 =>
                match_seq!($s, 1, $ensure_len,
                           0x80 ... 0x8F, 0x80 ... 0xBF, 0x80 ... 0xBF),
            _ => seq_error!()
        }
    }
}

macro_rules! partial_seq_or_check {
    (($s:expr, $seq_len:expr) $check_contents:block) => {
        if $s.len() < $seq_len {
            Ok(0)
        } else $check_contents
    }
}

macro_rules! whole_seq_check {
    (($s:expr, $seq_len:expr) $check_contents:block) => {
        $check_contents
    }
}

fn validate_next(s: &[u8]) -> Result<usize, DecodeError> {
    validate_next_impl!(s, partial_seq_or_check)
}

fn validate_next_bulk(s: &[u8]) -> Result<usize, DecodeError> {
    validate_next_impl!(s, whole_seq_check)
}

const UTF8_SEQ_LEN: [usize; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x00 ... 0x0F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x10 ... 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x20 ... 0x2F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x30 ... 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x40 ... 0x4F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x50 ... 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x60 ... 0x6F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x70 ... 0x7F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x80 ... 0x8F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0x90 ... 0x9F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0xA0 ... 0xAF
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,  // 0xB0 ... 0xBF
    1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,  // 0xC0 ... 0xCF
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,  // 0xD0 ... 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,  // 0xE0 ... 0xEF
    4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1   // 0xF0 ... 0xFF
];

impl Utf8 {
    fn take_partial(&mut self, input: &[u8]) -> usize {
        let have_len = self.stitch_len as usize;
        // Ensure, without branching, that self.stitch[0] is initialized
        // with the head of the currently accumulated sequence
        self.stitch[have_len] = input[0];
        let need_len = UTF8_SEQ_LEN[self.stitch[0] as usize];
        let partial_len = min(need_len - have_len, input.len());
        if partial_len != 0 {
            bytes::copy_memory(&mut self.stitch[have_len + 1
                                                .. have_len + partial_len],
                               &input[1 .. partial_len]);
            self.stitch_len = (have_len + partial_len) as u32;
        }
        partial_len
    }
}

impl Decode for Utf8 {
    fn decode<'a, 'b>(&'a mut self, input: &'b [u8])
                     -> decode::Result<Decoded<'a, 'b>>
    {
        if self.stitch_len != 0 {
            if input.len() == 0 {
                let out = self.output();
                if out.is_empty() {
                    return Err(DecodeError::new(
                        "input ends with an incomplete UTF-8 sequence"));
                }
                return Decoded::some(0, out);
            }
            // Try to complete the accumulated partial sequence
            let partial_len = self.take_partial(input);
            match validate_next(&self.stitch[.. self.stitch_len as usize]) {
                Ok(len) => {
                    let output = unsafe {
                        str::from_utf8_unchecked(&self.stitch[0 .. len])
                    };
                    return Decoded::some(partial_len, output);
                }
                Err(e) => {
                    self.stitch_len -= partial_len as u32;
                    return Err(e);
                }
            }
        }

        let mut i: usize = 0;
        while input.len() - i >= 4 {
            i += try!(validate_next_bulk(&input[i .. i + 4]));
        }
        while i < input.len() {
            let seq_len = try!(validate_next(&input[i ..]));
            if seq_len == 0 {
                if i != 0 {
                    // An incomplete sequence at the end,
                    // but there is some UTF-8 to return in place
                    break;
                }
                // An incomplete input sequence and nothing to output.
                // Accumulate it in the partial buffer.
                let partial_len = self.take_partial(input);
                return Decoded::some(partial_len, "");
            }
            i += seq_len;
        }
        Decoded::in_place(unsafe { str::from_utf8_unchecked(&input[.. i]) })
    }

    fn output(&self) -> &str {
        if self.stitch_len == 0 {
            return "";
        }
        let complete_len = UTF8_SEQ_LEN[self.stitch[0] as usize] as u32;
        if self.stitch_len != complete_len {
            return "";
        }
        let out = &self.stitch[0 .. (self.stitch_len as usize)];
        unsafe { str::from_utf8_unchecked(out) }
    }

    fn consume(&mut self) {
        self.stitch_len = 0;
    }

    fn reset(&mut self) {
        self.stitch_len = 0;
    }
}
