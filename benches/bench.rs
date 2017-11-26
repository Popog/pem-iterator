#![feature(test)]
#![cfg_attr(feature = "generators", feature(generators, generator_trait))]

extern crate test;
extern crate pem_iterator;
extern crate pem;
extern crate rand;

use std::iter::repeat;

#[cfg(feature = "generators")]
use std::ops::{Generator, GeneratorState};

use pem_iterator::boundary::{BoundaryType, BoundaryParser, LabelMatcher};
use pem_iterator::body::{Chunked, Single, ResultBytes};
#[cfg(feature = "generators")]
use pem_iterator::generator::{parse_boundary_chars, parse_body_chunked_chars,
                              parse_body_single_chars};
use test::{Bencher, black_box};
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


fn bench_chunked_collect(b: &mut Bencher, count: usize) {

    let s = gen(count);
    let mut label_buf = String::with_capacity(32);
    let mut vec: Vec<u8> = Vec::with_capacity(count / 4 * 3);

    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();
        vec.clear();
        {
            let mut parser =
                BoundaryParser::from_chars(BoundaryType::Begin, &mut input, &mut label_buf);
            assert_eq!(parser.next(), None);
            assert_eq!(parser.complete(), Ok(()));
        }

        for v in Chunked::from_chars(&mut input) {
            vec.extend(v.unwrap())
        }
        //let v: Result<Vec<u8>, _> = Chunked::from_chars(&mut input).collect();

        {
            let mut parser = BoundaryParser::from_chars(
                BoundaryType::End,
                &mut input,
                LabelMatcher(label_buf.chars()),
            );
            assert_eq!(parser.next(), None);
            assert_eq!(parser.complete(), Ok(()));
        }

        //v.unwrap()
    });
}

fn bench_chunked_iter(b: &mut Bencher, count: usize) {
    let s = gen(count);
    let mut label_buf = String::new();
    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();
        {
            let mut parser =
                BoundaryParser::from_chars(BoundaryType::Begin, &mut input, &mut label_buf);
            assert_eq!(parser.next(), None);
            assert_eq!(parser.complete(), Ok(()));
        }

        let v = Chunked::from_chars(&mut input).last().unwrap();

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
    });
}

fn bench_chunked_flat(b: &mut Bencher, count: usize) {
    let s = gen(count);
    let mut label_buf = String::new();
    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();

        {
            let mut parser =
                BoundaryParser::from_chars(BoundaryType::Begin, &mut input, &mut label_buf);
            assert_eq!(parser.next(), None);
            assert_eq!(parser.complete(), Ok(()));
        }

        let v = Chunked::from_chars(&mut input)
            .flat_map(ResultBytes)
            .last()
            .unwrap();

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
    });
}


fn bench_chunked_gen_iter(b: &mut Bencher, count: usize) {
    let s = gen(count);
    let mut label_buf = String::new();
    (&b, &s, &label_buf);

    #[cfg(feature = "generators")]
    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();
        {
            let mut gen = parse_boundary_chars(BoundaryType::Begin, input.by_ref(), &mut label_buf);
            match gen.resume() {
                GeneratorState::Yielded(_) => unreachable!(),
                GeneratorState::Complete(r) => assert_eq!(r, Ok(())),
            }
        }

        {
            let mut gen = parse_body_chunked_chars(input.by_ref());
            loop {
                match gen.resume() {
                    GeneratorState::Yielded(b) => b.unwrap(),
                    GeneratorState::Complete(_) => break,
                };
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
    });
}

fn bench_chunked_gen_collect(b: &mut Bencher, count: usize) {
    let s = gen(count);
    let mut label_buf = String::new();
    (&b, &s, &label_buf);

    #[cfg(feature = "generators")]
    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();
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
    });
}


fn bench_single_gen_iter(b: &mut Bencher, count: usize) {
    let s = gen(count);
    let mut label_buf = String::new();
    (&b, &s, &label_buf);

    #[cfg(feature = "generators")]
    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();
        {
            let mut gen = parse_boundary_chars(BoundaryType::Begin, input.by_ref(), &mut label_buf);
            match gen.resume() {
                GeneratorState::Yielded(_) => unreachable!(),
                GeneratorState::Complete(r) => assert_eq!(r, Ok(())),
            }
        }

        {
            let mut gen = parse_body_single_chars(input.by_ref());
            loop {
                match gen.resume() {
                    GeneratorState::Yielded(b) => b.unwrap(),
                    GeneratorState::Complete(_) => break,
                };
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
    });
}

fn bench_single_gen_collect(b: &mut Bencher, count: usize) {
    let s = gen(count);
    let mut label_buf = String::new();
    (&b, &s, &label_buf);

    #[cfg(feature = "generators")]
    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();
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
    });
}


fn bench_single_collect(b: &mut Bencher, count: usize) {
    let s = gen(count);
    let mut label_buf = String::new();
    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();
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
    });
}

fn bench_single_iter(b: &mut Bencher, count: usize) {
    let s = gen(count);
    let mut label_buf = String::new();
    b.iter(|| {
        let mut input = black_box(s.chars()).enumerate();
        label_buf.clear();
        {
            let mut parser =
                BoundaryParser::from_chars(BoundaryType::Begin, &mut input, &mut label_buf);
            assert_eq!(parser.next(), None);
            assert_eq!(parser.complete(), Ok(()));
        }

        let v = Single::from_chars(&mut input).last().unwrap();

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
    });
}


fn bench_pem(b: &mut Bencher, count: usize) {
    let s = gen(count);
    b.iter(|| { pem::parse(black_box(&s)).unwrap(); });
}

#[bench]
fn bench_a_100_a_pem(b: &mut Bencher) {
    bench_pem(b, 100)
}
#[bench]
fn bench_b_1000_a_pem(b: &mut Bencher) {
    bench_pem(b, 1000)
}
#[bench]
fn bench_c_10000_a_pem(b: &mut Bencher) {
    bench_pem(b, 10000)
}



#[bench]
fn bench_a_100_b_single_collect(b: &mut Bencher) {
    bench_single_collect(b, 100)
}
#[bench]
fn bench_b_1000_b_single_collect(b: &mut Bencher) {
    bench_single_collect(b, 1000)
}
#[bench]
fn bench_c_10000_b_single_collect(b: &mut Bencher) {
    bench_single_collect(b, 10000)
}

#[bench]
fn bench_a_100_c_single_iter(b: &mut Bencher) {
    bench_single_iter(b, 100)
}
#[bench]
fn bench_b_1000_c_single_iter(b: &mut Bencher) {
    bench_single_iter(b, 1000)
}
#[bench]
fn bench_c_10000_c_single_iter(b: &mut Bencher) {
    bench_single_iter(b, 10000)
}



#[bench]
fn bench_a_100_d_chunked_collect(b: &mut Bencher) {
    bench_chunked_collect(b, 100)
}
#[bench]
fn bench_b_1000_d_chunked_collect(b: &mut Bencher) {
    bench_chunked_collect(b, 1000)
}
#[bench]
fn bench_c_10000_d_chunked_collect(b: &mut Bencher) {
    bench_chunked_collect(b, 10000)
}

#[bench]
fn bench_a_100_e_chunked_flat(b: &mut Bencher) {
    bench_chunked_flat(b, 100)
}
#[bench]
fn bench_b_1000_e_chunked_flat(b: &mut Bencher) {
    bench_chunked_flat(b, 1000)
}
#[bench]
fn bench_c_10000_e_chunked_flat(b: &mut Bencher) {
    bench_chunked_flat(b, 10000)
}

#[bench]
fn bench_a_100_f_chunked_iter(b: &mut Bencher) {
    bench_chunked_iter(b, 100)
}
#[bench]
fn bench_b_1000_f_chunked_iter(b: &mut Bencher) {
    bench_chunked_iter(b, 1000)
}
#[bench]
fn bench_c_10000_f_chunked_iter(b: &mut Bencher) {
    bench_chunked_iter(b, 10000)
}



#[bench]
fn bench_a_100_g_chunked_gen_collect(b: &mut Bencher) {
    bench_chunked_gen_collect(b, 100)
}
#[bench]
fn bench_b_1000_g_chunked_gen_collect(b: &mut Bencher) {
    bench_chunked_gen_collect(b, 1000)
}
#[bench]
fn bench_c_10000_g_chunked_gen_collect(b: &mut Bencher) {
    bench_chunked_gen_collect(b, 10000)
}

#[bench]
fn bench_a_100_h_chunked_gen_iter(b: &mut Bencher) {
    bench_chunked_gen_iter(b, 100)
}
#[bench]
fn bench_b_1000_h_chunked_gen_iter(b: &mut Bencher) {
    bench_chunked_gen_iter(b, 1000)
}
#[bench]
fn bench_c_10000_h_chunked_gen_iter(b: &mut Bencher) {
    bench_chunked_gen_iter(b, 10000)
}



#[bench]
fn bench_a_100_i_single_gen_collect(b: &mut Bencher) {
    bench_single_gen_collect(b, 100)
}
#[bench]
fn bench_b_1000_i_single_gen_collect(b: &mut Bencher) {
    bench_single_gen_collect(b, 1000)
}
#[bench]
fn bench_c_10000_i_single_gen_collect(b: &mut Bencher) {
    bench_single_gen_collect(b, 10000)
}

#[bench]
fn bench_a_100_j_single_gen_iter(b: &mut Bencher) {
    bench_single_gen_iter(b, 100)
}
#[bench]
fn bench_b_1000_j_single_gen_iter(b: &mut Bencher) {
    bench_single_gen_iter(b, 1000)
}
#[bench]
fn bench_c_10000_j_single_gen_iter(b: &mut Bencher) {
    bench_single_gen_iter(b, 10000)
}
