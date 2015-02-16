// Copyright 2015 Mikhail Zabaluev <mikhail.zabaluev@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(core)]
#![feature(io)]
#![feature(unsafe_destructor)]

pub mod decode;
pub mod errors;
pub mod read;

pub mod decoders {
    pub use self::utf8::Utf8;

    mod utf8;
}

mod util;
