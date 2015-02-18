// Copyright 2015 Mikhail Zabaluev <mikhail.zabaluev@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use decode::Decode;
use util::VecMut;

use std::io;
use std::slice::bytes;

pub struct Reader<R, D> {
    reader: R,
    decoder: D,
    spillover: io::Cursor<Vec<u8>>
}

impl<R, D> Reader<R, D> where R: io::BufRead, D: Decode {
    pub fn new(reader: R, decoder: D) -> Reader<R, D> {
        Reader {
            reader: reader,
            decoder: decoder,
            spillover: io::Cursor::new(Vec::new())
        }
    }
}

impl<R, D> io::Read for Reader<R, D> where R: io::BufRead, D: Decode {

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.spillover.get_ref().is_empty() {
            let res_drain = self.spillover.read(buf);
            match res_drain {
                Ok(len) => {
                    if len != 0 {
                        return res_drain;
                    } else {
                        self.spillover.set_position(0);
                        self.spillover.get_mut().clear();
                    }
                }
                Err(e) => panic!(e)
            }
        }
        let (in_consumed, buf_filled) = {
            let buf_read = try!(self.reader.fill_buf());
            let decoded = try!(self.decoder.decode(buf_read));
            let out = decoded.output().as_bytes();
            let buf_filled = if out.len() < buf.len() {
                bytes::copy_memory(buf, out);
                out.len()
            } else {
                let (fit, rest) = out.split_at(buf.len());
                bytes::copy_memory(buf, fit);
                self.spillover.get_mut().extend(rest.iter().cloned());
                buf.len()
            };
            (decoded.input_len(), buf_filled)
        };
        self.reader.consume(in_consumed);
        Ok(buf_filled)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<()> {
        loop {
            let in_consumed = {
                let buf_read = try!(self.reader.fill_buf());
                let decoded = try!(self.decoder.decode(buf_read));
                buf.extend(decoded.output().bytes());
                let in_consumed = decoded.input_len();
                if in_consumed == 0 {
                    debug_assert!(buf_read.is_empty(),
                        "the decoder returned EOF on non-empty input");
                    break;
                }
                in_consumed
            };
            self.reader.consume(in_consumed);
        }
        Ok(())
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<()> {
        // Use the fact that successful reading always produces valid UTF-8
        let mut g = unsafe { VecMut::new(buf) };
        {
            let v = g.get_mut();
            self.read_to_end(v)
        }.and_then(|()| {
            unsafe { g.commit(); }
            Ok(())
        })
    }
}
