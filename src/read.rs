// Copyright 2015 Mikhail Zabaluev <mikhail.zabaluev@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use decode::{Decode, Decoded};

use std::cmp::min;
use std::io;
use std::io::{Read, BufRead};
use std::slice::bytes;
use std::str;

pub struct Reader<R, D> {
    reader: R,
    decoder: D,
    state: BufState
}

#[derive(Copy, Eq, PartialEq)]
enum BufState {
    Empty,
    Decoded { pos: usize },
    InPlace { len: usize }
}

impl<R, D> Reader<R, D> where R: BufRead, D: Decode {
    pub fn new(reader: R, decoder: D) -> Reader<R, D> {
        Reader { reader: reader, decoder: decoder, state: BufState::Empty }
    }

    fn read_to_end_fast_with<F>(&mut self, mut process: F) -> io::Result<()>
        where F: FnMut(&str)
    {
        debug_assert!(self.state == BufState::Empty);

        // Fast-pace trough the output, no need to change states
        loop {
            let (in_consumed, amt_read) = {
                let read_buf = try!(self.reader.fill_buf());
                let decoded = try!(self.decoder.decode(read_buf));
                match decoded {
                    Decoded::Some(in_consumed, out) => {
                        process(out);
                        (in_consumed, out.len())
                    }
                    Decoded::InPlace(s) => {
                        debug_assert!(s.as_ptr() == read_buf.as_ptr(),
                            "decoder returned in-place data not at the start of the input buffer");
                        process(s);
                        let in_len = s.len();
                        (in_len, in_len)
                    }
                }
            };
            self.reader.consume(in_consumed);
            self.decoder.consume();
            if amt_read == 0 {
                break;
            }
        }
        Ok(())
    }
}

impl<R, D> Read for Reader<R, D> where R: BufRead, D: Decode {

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = {
            let decoded_buf = try!(self.fill_buf());
            let amt = min(buf.len(), decoded_buf.len());
            bytes::copy_memory(&mut buf[0 .. amt], &decoded_buf[0 .. amt]);
            amt
        };
        self.consume(amt);
        Ok(amt)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let pre_len = buf.len();
        // Deal with possible partially read input
        let read_len = {
            let read_buf = try!(self.fill_buf());
            buf.extend(read_buf.iter().cloned());
            read_buf.len()
        };
        self.consume(read_len);
        try!(self.read_to_end_fast_with(|s| { buf.extend(s.bytes()) }));
        Ok(buf.len() - pre_len)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let pre_len = buf.len();
        // The reader might have been partially read from. Deal with it.
        {
            let partial_input: &[u8] = match self.state {
                BufState::Empty => &[],
                BufState::Decoded { pos } => {
                    &self.decoder.output().as_bytes()[pos ..]
                }
                BufState::InPlace { len } => {
                    let read_buf = try!(self.reader.fill_buf());
                    &read_buf[0 .. len]
                }
            };
            match str::from_utf8(partial_input) {
                Ok(s) => {
                    buf.push_str(s);
                }
                Err(_) => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput,
                        "cannot read a complete UTF-8 string due to preceding reads",
                        None));
                }
            }
        }
        self.decoder.consume();
        self.state = BufState::Empty;
        try!(self.read_to_end_fast_with(|s| { buf.push_str(s) }));
        Ok(buf.len() - pre_len)
    }
}

impl<R, D> BufRead for Reader<R, D> where R: BufRead, D: Decode {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        loop {
            match self.state {
                BufState::Empty => { }
                BufState::Decoded { pos } => {
                    let out = self.decoder.output();
                    return Ok(&out.as_bytes()[pos ..]);
                }
                BufState::InPlace { len } => {
                    // We had some validated UTF-8 in the input,
                    // and we should get the remaining part of it again
                    // from the buffered reader.
                    let buf = try!(self.reader.fill_buf());
                    return Ok(&buf[0 .. len]);
                }
            }
            let in_consumed = {
                let read_buf = try!(self.reader.fill_buf());
                let decoded = try!(self.decoder.decode(read_buf));
                match decoded {
                    Decoded::Some(in_consumed, _) => {
                        self.state = BufState::Decoded { pos: 0 };
                        in_consumed
                    }
                    Decoded::InPlace(s) => {
                        debug_assert!(s.as_ptr() == read_buf.as_ptr(),
                            "decoder returned in-place data not at the start of the input buffer");
                        self.state = BufState::InPlace { len: s.len() };
                        0
                    }
                }
            };
            self.reader.consume(in_consumed);
        }
    }

    fn consume(&mut self, amt: usize) {
        match self.state {
            BufState::Empty => {
                debug_assert!(amt == 0,
                    "request to consume exceeds the output");
            }
            BufState::Decoded { pos } => {
                let new_pos = pos + amt;
                let buf_len = self.decoder.output().len();
                debug_assert!(new_pos <= buf_len,
                    "request to consume exceeds the output");
                if new_pos == buf_len {
                    self.decoder.consume();
                    self.state = BufState::Empty;
                } else {
                    self.state = BufState::Decoded { pos: new_pos };
                }
            }
            BufState::InPlace { len } => {
                debug_assert!(amt <= len,
                    "request to consume exceeds the output");
                self.reader.consume(amt);
                let new_len = len - amt;
                self.state =
                    if new_len == 0 {
                        BufState::Empty
                    } else {
                        BufState::InPlace { len: new_len }
                    };
            }
        }
    }
}
