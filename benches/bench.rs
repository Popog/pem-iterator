#![feature(test)]

extern crate test;
extern crate pem_iterator;
extern crate pem;
extern crate rand;

use std::iter::repeat;

use pem_iterator::chunked::{Pem as Chunked, ResultBytes};
use pem_iterator::single::Pem as Single;
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


#[cfg(feature = "std")]
fn bench_chunked_collect(b: &mut Bencher, count: usize) {
    let s = gen(count);
    b.iter(|| {
        let pem = Chunked::from_chars(black_box(s.chars())).unwrap();
        let _: Result<Vec<u8>, _> = pem.collect();
    });
}

#[cfg(not(feature = "std"))]
fn bench_chunked_collect(b: &mut Bencher, count: usize) {
    let s = gen(count);
    b.iter(|| {
        let pem = Chunked::from_chars(black_box(s.chars())).unwrap();
        let _: Result<Vec<u8>, _> = pem.flat_map(ResultBytes).collect();
    });
}

fn bench_chunked_iter(b: &mut Bencher, count: usize) {
    let s = gen(count);
    b.iter(|| {
        let pem = Chunked::from_chars(black_box(s.chars())).unwrap();
        pem.last().unwrap().unwrap();
    });
}

fn bench_chunked_flat(b: &mut Bencher, count: usize) {
    let s = gen(count);
    b.iter(|| {
        let pem = Chunked::from_chars(black_box(s.chars())).unwrap();
        pem.flat_map(ResultBytes).last().unwrap().unwrap();
    });
}


fn bench_single_collect(b: &mut Bencher, count: usize) {
    let s = gen(count);
    b.iter(|| {
        let pem = Single::from_chars(black_box(s.chars())).unwrap();
        let _: Result<Vec<u8>, _> = pem.collect();
    });
}

fn bench_single_iter(b: &mut Bencher, count: usize) {
    let s = gen(count);
    b.iter(|| {
        let pem = Single::from_chars(black_box(s.chars())).unwrap();
        pem.last().unwrap().unwrap();
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
