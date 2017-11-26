#![cfg_attr(feature = "generators", feature(generators, generator_trait))]

extern crate pem_iterator;
extern crate pem;
extern crate rand;

use std::iter::repeat;

#[cfg(feature = "generators")]
use std::ops::{Generator, GeneratorState};

use pem_iterator::boundary::{BoundaryType, BoundaryParser, LabelMatcher};
use pem_iterator::body::{Chunked, Single};
#[cfg(not(feature = "std"))]
use pem_iterator::body::BytesContainer;
#[cfg(feature = "generators")]
use pem_iterator::generator::{parse_boundary_chars, parse_body_chunked_chars,
                              parse_body_single_chars};
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



fn chunked(s: &str) -> Vec<u8> {

    let mut input = s.chars().enumerate();

    let mut label_buf = String::new();
    {
        let mut parser =
            BoundaryParser::from_chars(BoundaryType::Begin, &mut input, &mut label_buf);
        assert_eq!(parser.next(), None);
        assert_eq!(parser.complete(), Ok(()));
    }

    #[cfg(feature = "std")]
    let v: Result<Vec<u8>, _> = Chunked::from_chars(&mut input).collect();

    #[cfg(not(feature = "std"))]
    let v: Result<BytesContainer<Vec<u8>>, _> = Chunked::from_chars(&mut input).collect();

    {
        let mut parser = BoundaryParser::from_chars(
            BoundaryType::End,
            &mut input,
            LabelMatcher(label_buf.chars()),
        );
        assert_eq!(parser.next(), None);
        assert_eq!(parser.complete(), Ok(()));
    }

    v.unwrap().into()
}

fn single(s: &str) -> Vec<u8> {

    let mut input = s.chars().enumerate();

    let mut label_buf = String::new();
    {
        let mut parser =
            BoundaryParser::from_chars(BoundaryType::Begin, &mut input, &mut label_buf);
        assert_eq!(parser.next(), None);
        assert_eq!(parser.complete(), Ok(()));
    }

    let v: Result<Vec<u8>, _> = Single::from_chars(&mut input).collect();

    {
        let mut parser = BoundaryParser::from_chars(
            BoundaryType::End,
            &mut input,
            LabelMatcher(label_buf.chars()),
        );
        assert_eq!(parser.next(), None);
        assert_eq!(parser.complete(), Ok(()));
    }

    v.unwrap()
}

#[cfg(feature = "generators")]
fn single_gen(s: &str) -> Vec<u8> {
    let mut input = s.chars().enumerate();
    let mut label_buf = String::new();
    {
        let mut gen = parse_boundary_chars(BoundaryType::Begin, input.by_ref(), &mut label_buf);
        match gen.resume() {
            GeneratorState::Yielded(_) => unreachable!(),
            GeneratorState::Complete(r) => assert_eq!(r, Ok(())),
        }
    }
    let mut v: Vec<u8> = vec![];
    {
        let mut gen = parse_body_single_chars(input.by_ref());
        loop {
            match gen.resume() {
                GeneratorState::Yielded(b) => v.push(b.unwrap()),
                GeneratorState::Complete(_) => break,
            }
        }
    }

    {
        let mut gen = parse_boundary_chars(
            BoundaryType::End,
            input.by_ref(),
            LabelMatcher(label_buf.chars()),
        );
        match gen.resume() {
            GeneratorState::Yielded(_) => unreachable!(),
            GeneratorState::Complete(r) => assert_eq!(r, Ok(())),
        }
    }
    v
}

#[cfg(feature = "generators")]
fn chunked_gen(s: &str) -> Vec<u8> {
    let mut input = s.chars().enumerate();
    let mut label_buf = String::new();
    {
        let mut gen = parse_boundary_chars(BoundaryType::Begin, input.by_ref(), &mut label_buf);
        match gen.resume() {
            GeneratorState::Yielded(_) => unreachable!(),
            GeneratorState::Complete(r) => assert_eq!(r, Ok(())),
        }
    }
    let mut v: Vec<u8> = vec![];
    {
        let mut gen = parse_body_chunked_chars(input.by_ref());
        loop {
            match gen.resume() {
                GeneratorState::Yielded(b) => v.extend(b.unwrap()),
                GeneratorState::Complete(_) => break,
            }
        }
    }

    {
        let mut gen = parse_boundary_chars(
            BoundaryType::End,
            input.by_ref(),
            LabelMatcher(label_buf.chars()),
        );
        match gen.resume() {
            GeneratorState::Yielded(_) => unreachable!(),
            GeneratorState::Complete(r) => assert_eq!(r, Ok(())),
        }
    }
    v
}

fn pem(s: &str) -> Vec<u8> {
    pem::parse(&s).unwrap().contents
}


fn test(count: usize) {
    let s = gen(count);
    let single = single(s.as_str());
    #[cfg(feature = "generators")]
    let single_gen = single_gen(s.as_str());
    let chunked = chunked(s.as_str());
    #[cfg(feature = "generators")]
    let chunked_gen = chunked_gen(s.as_str());
    let pem = pem(s.as_str());
    #[cfg(feature = "generators")]
    assert_eq!(single, single_gen);
    assert_eq!(single, chunked);
    #[cfg(feature = "generators")]
    assert_eq!(single, chunked_gen);
    assert_eq!(single, pem);
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
