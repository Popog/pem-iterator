extern crate pem_iterator;
extern crate pem;
extern crate rand;

use std::iter::repeat;

use pem_iterator::chunked::Pem as Chunked;
use pem_iterator::single::Pem as Single;
use rand::{Rng, weak_rng};


fn gen(count: usize) -> String {
    let mut rng = weak_rng();
    let v = [
        'A',
        'B',
        'C',
        'D',
        'E',
        'F',
        'G',
        'H',
        'I',
        'J',
        'K',
        'L',
        'M',
        'N',
        'O',
        'P',
        'Q',
        'R',
        'S',
        'T',
        'U',
        'V',
        'W',
        'X',
        'Y',
        'Z',
        'a',
        'b',
        'c',
        'd',
        'e',
        'f',
        'g',
        'h',
        'i',
        'j',
        'k',
        'l',
        'm',
        'n',
        'o',
        'p',
        'q',
        'r',
        's',
        't',
        'u',
        'v',
        'w',
        'x',
        'y',
        'z',
        '0',
        '1',
        '2',
        '3',
        '4',
        '5',
        '6',
        '7',
        '8',
        '9',
        '+',
        '/',
    ];

    "-----BEGIN DATA-----"
        .chars()
        .chain(repeat(()).take(count).map(|_| v[rng.gen_range(0, 64)]))
        .chain("-----END DATA-----".chars())
        .collect()
}



#[cfg(feature = "std")]
fn chunked(s: &str) -> Vec<u8> {
    let pem = Chunked::from_chars(s.chars()).unwrap();
    let v: Result<Vec<u8>, _> = pem.collect();
    v.unwrap()
}

#[cfg(not(feature = "std"))]
fn chunked(s: &str) -> Vec<u8> {
    use pem_iterator::chunked::ResultBytes;
    let pem = Chunked::from_chars(s.chars()).unwrap();
    let v: Result<Vec<u8>, _> = pem.flat_map(ResultBytes).collect();
    v.unwrap()
}

fn single(s: &str) -> Vec<u8> {
    let pem = Single::from_chars(s.chars()).unwrap();
    let v: Result<Vec<u8>, _> = pem.collect();
    v.unwrap()
}


fn pem(s: &str) -> Vec<u8> {
    pem::parse(&s).unwrap().contents
}


fn test(count: usize) {
    let s = gen(count);
    let single = single(s.as_str());
    let chunked = chunked(s.as_str());
    let pem = pem(s.as_str());
    assert_eq!(single, chunked);
    assert_eq!(single, pem);
    assert_eq!(pem, chunked);
}

#[test]
fn test_100() {
    test(100)
}
#[test]
fn test_1000() {
    test(1000)
}
#[test]
fn test_10000() {
    test(10000)
}
