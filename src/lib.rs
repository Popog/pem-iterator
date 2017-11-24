#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

#[cfg(all(feature = "store_label", not(feature = "std")))]
use core::fmt;

mod parse;
mod errors;
pub mod chunked;
pub mod single;


pub use chunked::Pem as Chunked;
pub use single::Pem as Single;
pub use errors::{ParseError, PemError, PreEncapsulationBoundaryError, Void};


fn inc(v: &mut usize) -> usize {
    let i = *v;
    *v += 1;
    i
}

#[cfg(not(feature = "store_label"))]
type Label = ();

#[cfg(all(feature = "store_label", feature = "std"))]
use std::string::String as Label;

#[cfg(all(feature = "store_label", not(feature = "std")))]
const MAX_LABEL_SIZE: usize = 64;

#[cfg(all(feature = "store_label", not(feature = "std")))]
pub struct Label {
    content: [u8; MAX_LABEL_SIZE],
    len: usize,
}

#[cfg(all(feature = "store_label", not(feature = "std")))]
impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

#[cfg(all(feature = "store_label", not(feature = "std")))]
impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

#[cfg(all(feature = "store_label", not(feature = "std")))]
impl Label {
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.content[0..self.len]).unwrap()
    }

    fn add(&mut self, c: char) -> bool {
        if self.len + c.len_utf8() > MAX_LABEL_SIZE {
            false
        } else {
            self.len += c.encode_utf8(&mut self.content[self.len..]).len();
            true
        }
    }
}
