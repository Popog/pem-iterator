#![cfg_attr(feature = "generators", feature(generators, generator_trait, conservative_impl_trait))]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

pub mod body;
pub mod boundary;

#[cfg(feature = "generators")]
pub mod generator;

/// Will be replaced with never_type `!` (rust/rust-lang#35121) 
#[derive(Clone, Copy, Debug, PartialEq, Eq)] 
pub enum Void {}

fn map_chars<Loc>(c: (Loc, char)) -> Result<(Loc, char), Void> {
    Ok(c)
}
 
#[cfg(feature = "std")] 
fn is_whitespace<Loc>(&(_, ref c): &(Loc, char)) -> bool { 
    c.is_whitespace() 
}
 
#[cfg(not(feature = "std"))] 
fn is_whitespace<Loc>(&(_, ref c): &(Loc, char)) -> bool { 
    fn trie_range_leaf(c: u32, bitmap_chunk: u64) -> bool { 
        ((bitmap_chunk >> (c & 63)) & 1) != 0 
    } 
 
    pub struct SmallBoolTrie { 
        r1: [u8; 193], // first level 
        r2: [u64; 6], // leaves 
    } 
 
    impl SmallBoolTrie { 
        fn lookup(&self, c: char) -> bool { 
            let c = c as u32; 
            match self.r1.get((c >> 6) as usize) { 
                Some(&child) => trie_range_leaf(c, self.r2[child as usize]), 
                None => false, 
            } 
        } 
    } 
 
    const WHITE_SPACE_TABLE: &'static SmallBoolTrie = &SmallBoolTrie { 
        r1: [ 
            0, 
            1, 
            2, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            3, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            4, 
            5, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            1, 
            3, 
        ], 
        r2: [ 
            0x0000000100003e00, 
            0x0000000000000000, 
            0x0000000100000020, 
            0x0000000000000001, 
            0x00008300000007ff, 
            0x0000000080000000, 
        ], 
    }; 
 
    WHITE_SPACE_TABLE.lookup(*c) 
} 
