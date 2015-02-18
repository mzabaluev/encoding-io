// Copyright 2015 Mikhail Zabaluev <mikhail.zabaluev@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::mem::transmute;

pub struct VecMut<'a> {
    vec_ref: &'a mut Vec<u8>,
    safe_len: usize
}

impl<'a> VecMut<'a> {
    pub unsafe fn new(s: &'a mut String) -> VecMut<'a> {
        let ptr = s.as_ptr() as *mut _;
        let len = s.len();
        let cap = s.capacity();
        let v: &'a mut Vec<u8> = transmute(s);
        *v = Vec::from_raw_parts(ptr, len, cap);
        VecMut { vec_ref: v, safe_len: len }
    }

    pub fn get_mut(&mut self) -> &mut Vec<u8> {
        self.vec_ref
    }

    pub unsafe fn commit(&mut self) {
        let s = {
            let v = self.get_mut();
            let ptr = v.as_ptr() as *mut _;
            let len = v.len();
            let cap = v.capacity();
            let s: &'a mut String = transmute(v);
            *s = String::from_raw_parts(ptr, len, cap);
            s
        };
        *self = VecMut::new(s);
    }
}

#[unsafe_destructor]
impl<'a> Drop for VecMut<'a> {
    fn drop(&mut self) {
        let len = self.safe_len;
        {
            let v = self.get_mut();
            unsafe { v.set_len(len); }
        }
        unsafe { self.commit() };
    }
}
