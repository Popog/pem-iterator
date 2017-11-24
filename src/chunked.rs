use core::marker::PhantomData;
use core::iter::Map;
use core::borrow::{Borrow, BorrowMut};
use core::slice;

#[cfg(feature = "std")]
use core::iter::FromIterator;

use {PreEncapsulationBoundaryError, Label, PemError, Void, inc};
use parse::{SourceError, LabelCharacters, expect, expect_begin, expect_label, get_6_bits};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bytes {
    One([u8; 1]),
    Two([u8; 2]),
    Three([u8; 3]),
}

pub struct ResultBytes<E>(pub Result<Bytes, PemError<E>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BytesIter {
    Zero,
    One([u8; 1]),
    Two([u8; 2]),
    Three([u8; 3]),
}

pub enum ResultBytesIter<E> {
    Empty,
    Err(PemError<E>),
    One([u8; 1]),
    Two([u8; 2]),
    Three([u8; 3]),
}

#[cfg(feature = "std")]
impl Extend<Bytes> for Vec<u8> {
    #[inline]
    fn extend<T: IntoIterator<Item = Bytes>>(&mut self, iter: T) {
        for bytes in iter {
            match bytes {
                Bytes::One(a) => self.push(a[0]),
                Bytes::Two(a) => self.extend_from_slice(&a[..]),
                Bytes::Three(a) => self.extend_from_slice(&a[..]),
            }
        }
    }
}

#[cfg(feature = "std")]
impl FromIterator<Bytes> for Vec<u8> {
    fn from_iter<T: IntoIterator<Item = Bytes>>(iter: T) -> Self {
        let mut map = Vec::new();
        map.extend(iter);
        map
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            &Bytes::One(ref a) => a,
            &Bytes::Two(ref a) => a,
            &Bytes::Three(ref a) => a,
        }
    }
}
impl AsMut<[u8]> for Bytes {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            &mut Bytes::One(ref mut a) => a,
            &mut Bytes::Two(ref mut a) => a,
            &mut Bytes::Three(ref mut a) => a,
        }
    }
}

impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        match self {
            &Bytes::One(ref a) => a,
            &Bytes::Two(ref a) => a,
            &Bytes::Three(ref a) => a,
        }
    }
}

impl BorrowMut<[u8]> for Bytes {
    fn borrow_mut(&mut self) -> &mut [u8] {
        match self {
            &mut Bytes::One(ref mut a) => a,
            &mut Bytes::Two(ref mut a) => a,
            &mut Bytes::Three(ref mut a) => a,
        }
    }
}

impl IntoIterator for Bytes {
    type Item = u8;
    type IntoIter = BytesIter;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Bytes::One(a) => BytesIter::One(a),
            Bytes::Two(a) => BytesIter::Two(a),
            Bytes::Three(a) => BytesIter::Three(a),
        }
    }
}

impl<'a> IntoIterator for &'a Bytes {
    type Item = &'a u8;
    type IntoIter = slice::Iter<'a, u8>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Bytes {
    type Item = &'a mut u8;
    type IntoIter = slice::IterMut<'a, u8>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_mut().into_iter()
    }
}

impl<E> IntoIterator for ResultBytes<E> {
    type Item = Result<u8, PemError<E>>;
    type IntoIter = ResultBytesIter<E>;
    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Err(e) => ResultBytesIter::Err(e),
            Ok(Bytes::One(a)) => ResultBytesIter::One(a),
            Ok(Bytes::Two(a)) => ResultBytesIter::Two(a),
            Ok(Bytes::Three(a)) => ResultBytesIter::Three(a),
        }
    }
}


impl Iterator for BytesIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            BytesIter::Zero => None,
            BytesIter::One(a) => {
                *self = BytesIter::Zero;
                Some(a[0])
            }
            BytesIter::Two(a) => {
                *self = BytesIter::One([a[1]]);
                Some(a[0])
            }
            BytesIter::Three(a) => {
                *self = BytesIter::Two([a[1], a[2]]);
                Some(a[0])
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            &BytesIter::Zero => (0, Some(0)),
            &BytesIter::One(..) => (1, Some(1)),
            &BytesIter::Two(..) => (2, Some(2)),
            &BytesIter::Three(..) => (3, Some(3)),
        }
    }
}

impl ExactSizeIterator for BytesIter {}


impl<E> Iterator for ResultBytesIter<E> {
    type Item = Result<u8, PemError<E>>;
    fn next(&mut self) -> Option<Self::Item> {
        use core::mem::replace;
        match replace(self, ResultBytesIter::Empty) {
            ResultBytesIter::Empty => None,
            ResultBytesIter::Err(e) => Some(Err(e)),
            ResultBytesIter::One(a) => Some(Ok(a[0])),
            ResultBytesIter::Two(a) => {
                *self = ResultBytesIter::One([a[1]]);
                Some(Ok(a[0]))
            }
            ResultBytesIter::Three(a) => {
                *self = ResultBytesIter::Two([a[1], a[2]]);
                Some(Ok(a[0]))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            &ResultBytesIter::Empty => (0, Some(0)),
            &ResultBytesIter::Err(..) => (1, Some(1)),
            &ResultBytesIter::One(..) => (1, Some(1)),
            &ResultBytesIter::Two(..) => (2, Some(2)),
            &ResultBytesIter::Three(..) => (3, Some(3)),
        }
    }
}

impl<E> ExactSizeIterator for ResultBytesIter<E> {}


pub struct Pem<E, R> {
    label: Label,
    characters: LabelCharacters,
    flag: bool,
    _marker: PhantomData<*const E>,
    count: usize,
    stream: R,
}

impl<R> Pem<Void, Map<R, fn(char) -> Result<char, Void>>>
where
    R: Iterator<Item = char>,
{
    pub fn from_chars(stream: R) -> Result<Self, PreEncapsulationBoundaryError<Void>> {
        fn convert(c: char) -> Result<char, Void> {
            Ok(c)
        }
        Pem::new(stream.map(convert))
    }
}


impl<E, R> Pem<E, R>
where
    R: Iterator<Item = Result<char, E>>,
{
    /// Create a PEM parser that parses up to 3 bytes of output at at time
    pub fn new(mut stream: R) -> Result<Self, PreEncapsulationBoundaryError<E>> {
        let mut count: usize = 0;
        let (label, characters) = {
            let mut stream = stream.by_ref().map(|r|
                r.map(|v| (inc(&mut count), v)).map_err(SourceError)
            );

            expect_begin(&mut stream)??;

            // get the label (this will eat the first two '-'s of the header end)
            let (label, characters) = expect_label(&mut stream)??;

            // eat the rest of the header
            expect(&mut "---".chars(), &mut stream)??;

            (label, characters)
        };

        Ok(Pem {
            label,
            characters,
            stream,
            flag: false,
            _marker: PhantomData,
            count,
        })
    }

    fn expect_footer(&mut self) -> Result<(), PemError<E>> {
        let count = &mut self.count;
        let mut stream = self.stream.by_ref().map(|r|
            r.map(|v| (inc(count), v)).map_err(SourceError)
        );

        #[cfg(feature = "store_label")]
        {
            expect(&mut "----END ".chars(), &mut stream)??;
            expect(&mut self.label.as_str().chars(), &mut stream)??;
            expect(&mut "-----".chars(), &mut stream)??;
        }

        #[cfg(not(feature = "store_label"))]
        {
            expect(&mut "----END ".chars(), &mut stream)??;
            expect_label(&mut stream)??;
            expect(&mut "---".chars(), &mut stream)??;
        }

        Ok(())
    }

    fn get_24_bits(&mut self) -> Result<Result<Bytes, Option<Bytes>>, PemError<E>> {
        let count = &mut self.count;
        let mut stream = self.stream.by_ref().map(|r|
            r.map(|v| (inc(count), v)).map_err(SourceError)
        );

        // Ignore whitespace
        let a = if let Some(a) = get_6_bits(&mut stream)? {
            a << 2
        } else {
            return Ok(Err(None));
        };

        let (a, b) = if let Some(b) = get_6_bits(&mut stream)? {
            (a | (b >> 4), (b & 0b1111) << 4)
        } else {
            return Ok(Err(Some(Bytes::One([a]))));
        };

        let (b, c) = if let Some(c) = get_6_bits(&mut stream)? {
            (b | (c >> 2), (c & 0b11) << 6)
        } else {
            return Ok(Err(Some(Bytes::Two([a, b]))));
        };

        let c = if let Some(d) = get_6_bits(&mut stream)? {
            c | d
        } else {
            return Ok(Err(Some(Bytes::Three([a, b, c]))));
        };

        Ok(Ok(Bytes::Three([a, b, c])))
    }

    #[cfg(feature = "store_label")]
    /// Get the label for the PEM
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Borrows the stream
    pub fn stream(&mut self) -> &mut R {
        &mut self.stream
    }

    /// Replaces the stream with another stream.
    pub fn replace_stream<E2, R2, F>(self, f: F) -> Pem<E2, R2>
    where
        R2: Iterator<Item = Result<char, E2>>,
        F: FnOnce(R) -> R2,
    {
        let Pem {
            label,
            characters,
            stream,
            flag,
            count,
            _marker,
        } = self;
        let stream = f(stream);
        Pem {
            label,
            characters,
            stream,
            flag,
            count,
            _marker: PhantomData,
        }
    }

    /// Unwrap the label and the iterator
    ///
    /// It is recommended to `collect` the iterator first, otherwise the post-encapsulation
    /// boundary won't be consumed
    pub fn into_inner(self) -> (Label, R) {
        (self.label, self.stream)
    }
}

impl<E, R> Iterator for Pem<E, R>
where
    R: Iterator<Item = Result<char, E>>,
{
    type Item = Result<Bytes, PemError<E>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.flag {
            return None;
        }

        match self.get_24_bits() {
            Err(e) => {
                self.flag = true;
                Some(Err(e))
            }
            Ok(Ok(b)) => Some(Ok(b)),
            Ok(Err(b)) => {
                self.flag = true;
                if let Err(e) = self.expect_footer() {
                    Some(Err(e))
                } else if let Some(b) = b {
                    Some(Ok(b))
                } else {
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // min is always 0 because we might be at the end
        (
            0,
            if self.flag {
                Some(0)
            } else {
                // The case where we yield the most is:
                // every remaining character is base64
                //
                // In this scenario every 4 characters will be turned into 1 result (rounded up)
                // and then we will yield 1 error because there is no footer
                self.stream.size_hint().1.map(|x| (x + 3) / 4 + 1)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use {PreEncapsulationBoundaryError, PemError, ParseError};
    use super::Pem;

    #[test]
    fn simple() {
        const PRIVATE: &'static str = "-----BEGIN RSA PRIVATE KEY-----
tW7bcIGQT1Gh2IIgglXL8yHoQfk1MYBah8B5NX2QSvFSfbpXfAhJYybmUd02sfOh
E5AvZsH9+dYL1Fxy9itlxkxC0i26ZNxr0qD0x+VesQCE2hhYW/YIfcQuFWm6ewps
5rjEmM1HlornVDkFcs0IepeJuRQ8w4HjjSCRLtWkpp6eiQ/KanNAj8aFmf/ganfz
sEFqY4d8G29Xw+oPNlhNYeG7YdD+cdzI95N0XR7tq2xr34N97DP3vMKskasQ4q+W
qy+3vfSDjSnqbOv5OqoxQ2iBzaBYmtZrCIe+1UVBj9YhPmnD5PtP+RVEx2Q7u3AJ
eQByEcFXjUpT1gBC61MBjno+TD/FV/1VEVmcljHHLGKnbq/UaPmqK0xniX9Komsh
ZS2ecM35HDThFcGBOUhlJxOSe9iCzw1PgII05Jjeanq1v2+FM8j7IdohNIRFYeQk
XAgGVYIMWXG41AJAJwQ3wfdP+kGOEWnebA/3lNRxYqrQzernexN/v6VAU1C8bxsS
iwvbPyhYEqEGgt0CV02m+q4FhgnUz8QKmI/+8CF4jv07FfDldWHmEMWLXreb02M8
MegqlyRPZUVE35RqvhnTNgrvgRSI8Y2TQlCfZad40y1cU7Oa2dJsJn/IuvT7KpfG
E57s4Ms1oD8FhUhOfF0W9okFC990jTawS+v/qaeWCbIYgla2zRK20Auf3OJ8dWlW
9Oz9YCdYtEFrJXXG26h9F7aPTMQiInQKW1tHYMiwP6WJmRn9Jg6NGk8dfZuVaTzQ
OC54KUThwRKd8co428e4Nrop36ffsKPILZzlvAtunLVVbeIiFcmEs6gJQA9lla4Z
1HaDtFiG9ujZKR7Sa93xpQ7xSYArfYdjnfteQ/8Q3D/iTMxn0k/orj9q+PmDtyWQ
vaj0q09bqB+awGxgJpNnwjtCYCiYTQCSWSR2zVlHgEHTLbSqF/Qbi8NxagVUIiKO
QMgwQwJYyiA8A+0brhw6WnFX1kTb7YAdSJp4HSFaquaBb1icEdb6KEujFtP3SqS2
eMOCF9H/WBlVL/vu/nfOAoxfy6iXN5GTkuWBEeYCq5MdmBnUBcHJmIOLy/bdtJ/c
DWHv7c7TH25xlI0pQvTa913G6BkKYPjkDSX4+pwtQwDvXp+knwMAnt95B1B+hjBX
IRpLtZ2Z0XEShZr13sNCnLAF4/OARi82afzGZxk0gnQZgCVEoGYzf5VHH9c7a5UX
2T5E4qIrbDJNTeqMUJ2maZQgtsrIvoTA4KJc97vka0sUgTR1j46ugSYsYSjdZSHZ
QhT3JVmYxVFwoW74aCKDLVrhAp35NuSgOywYkJCkacxotoZvn15crGxnoQPw0+GZ
fDaBkVss1H/u1ZxcxCt4OA==
-----END RSA PRIVATE KEY-----";

        const INT_CERT: &'static str = "-----BEGIN INTERMEDIATE CERT-----
D7NCjd+Y3I6b1OR22wuh04us+70pwDEBDxuqT6QisvPED19w7WqRjhjxdOyN+qR8
rClp8X8ZIvnrmmgDkb2NvAyIjtt5BsOsKm4S8Ra930pOe5OvPqE4oOhIvMIL0dny
irGMLZ8oUpQU85cZgZjmWhz+ZSkZCM059+4z8LJEL1wXsqQjpTudAO8PyMPx9ZZD
2Pl1jeWU8tWyyT4iY47QxOBpfi/TaYDopiKCvZ29GiQpbamrjO3RPWl4nE4sn2Wd
V48X/0kEKsAz0NVTtIfMuRxlZllTsmJsa4NKPW3S/erc2hh/OnYLNqCMkids/Mwz
Bs1s4bB/RaJxj2/u6sP75Q==
-----END INTERMEDIATE CERT-----";

        const CERT: &'static str = "-----BEGIN CERTIFICATE-----
4q2QGsqDrji0tYif1qMHY3iJfvtDC44BfqDKMOdgQBREl+U6pPn2WTxnvUms8rSV
o3GBQqw6YhffKHbMGtU6my+KHWmPFBHclgrHaRL8TD+jxUY6YMmzXbC9cBaqN455
wQxauHWbnIPo9r9Bn0UjQ7PIx/GWD006YHF4AcFHRKwuZVbXsZBpmYUkgDMZ7MpC
uc94FQ7xYbLKfKQ6/MNJQ1WHmLuhrFs6vhR/KXcgnaJ/y0cSMTLUaHHl7S0R8yiE
IjlUHwyqg2uGDUXvIVeUfj1wOmjFSQBRRsmCu8zvfc4pM5svg4nfKJY4x+DjjaaY
UrBu8KYoMHMPhsEAnhSBum2jie3y72w2bgdxTQYbSJmoeVCT0UOKkuHBwNh7MSe3
O4bKdG1UzqhXJulr
-----END CERTIFICATE-----";

        fn helper(input: &str, label: &str) {
            let mut pem = Pem::from_chars(input.chars()).unwrap();

            #[cfg(feature = "store_label")]
            assert_eq!(pem.label(), label);
            let _ = label;

            assert_eq!(None, pem.find(|v| v.is_err()));
        }
        helper(CERT, "CERTIFICATE");
        helper(INT_CERT, "INTERMEDIATE CERT");
        helper(PRIVATE, "RSA PRIVATE KEY");
    }

    #[test]
    fn test_parse_pre_only() {
        fn helper(input: &str) {
            let mut pem = Pem::from_chars(input.chars()).unwrap();

            #[cfg(feature = "store_label")]
            assert_eq!(pem.label(), "CERTIFICATE");

            assert_eq!(
                pem.next(),
                Some(Err(PemError::ParseError(ParseError::MissingExpected('-'))))
            );
        }

        helper("-----BEGIN CERTIFICATE-----");
        helper("  \n\r\t-----BEGIN CERTIFICATE-----");
        helper("  \n\r\t-----BEGIN CERTIFICATE-----\n\t\r  ");
        helper("-----BEGIN CERTIFICATE-----\n\t\r  ");
    }

    #[test]
    fn test_parse_invalid_framing() {
        assert_eq!(
            Pem::from_chars("--BEGIN data----".chars()).err().unwrap(),
            PreEncapsulationBoundaryError::Mismatch {
                expected: '-',
                found: 'B',
                location: 2,
            }
        );
    }

    #[test]
    fn test_parse_empty_data() {
        fn helper(input: &str, label: &str) {
            let mut pem = Pem::from_chars(input.chars()).unwrap();

            #[cfg(feature = "store_label")]
            assert_eq!(pem.label(), label);
            let _ = label;

            assert_eq!(pem.next(), None);
        }
        helper("-----BEGIN DATA-----\r-----END DATA-----", "DATA");
        helper("-----BEGIN DATA----------END DATA-----", "DATA");
    }

    #[cfg(feature = "store_label")]
    #[test]
    fn mismatch_label() {
        let input = "-----BEGIN DATA-----\n-----END CONFIG-----";
        let mut pem = Pem::from_chars(input.chars()).unwrap();

        assert_eq!(pem.label(), "DATA");

        assert_eq!(
            pem.next(),
            Some(Err(PemError::ParseError(ParseError::Mismatch {
                expected: 'D',
                found: 'C',
                location: 30,
            })))
        );
    }

    #[cfg(not(feature = "store_label"))]
    #[test]
    fn mismatch_label() {
        let input = "-----BEGIN DATA-----\n-----END CONFIG-----";
        let mut pem = Pem::from_chars(input.chars()).unwrap();
        assert_eq!(pem.next(), None);
    }
}
