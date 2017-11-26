use core::iter::Map;

use core::borrow::{Borrow, BorrowMut};
use core::slice;
 
use core::iter::FromIterator;

use {Void, map_chars, is_whitespace};

#[derive(Debug, PartialEq)]
pub enum BodyError<Loc, E> {
    InvalidCharacter{
        location: Loc,
        found: char
    },
    MissingExpected(char),
    SourceError(E),
}

impl<Location, E> From<E> for BodyError<Location, E> {
    fn from(e: E) -> Self {
        BodyError::SourceError(e)
    }
}




#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bytes {
    One([u8; 1]),
    Two([u8; 2]),
    Three([u8; 3]),
}

pub struct ResultBytes<Loc, E>(pub Result<Bytes, BodyError<Loc, E>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BytesIter {
    Zero,
    One([u8; 1]),
    Two([u8; 2]),
    Three([u8; 3]),
}

pub enum ResultBytesIter<Loc, E> {
    Empty,
    Err(BodyError<Loc, E>),
    One([u8; 1]),
    Two([u8; 2]),
    Three([u8; 3]),
}

// A helper for collecting Bytes into Container<u8>
pub struct BytesContainer<T>(pub T);

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
        let mut vec = Vec::new();
        vec.extend(iter);
        vec
    }
}

impl<C> BytesContainer<C> {
    pub fn into(self) -> C {
        self.0
    }
}

impl<C> Extend<Bytes> for BytesContainer<C>
where C: Extend<u8> {
    #[inline]
    fn extend<T: IntoIterator<Item = Bytes>>(&mut self, iter: T) {
        use core::iter::once;
        for bytes in iter {
            match bytes {
                Bytes::One(a) => self.0.extend(once(a[0])),
                Bytes::Two(a) => self.0.extend(once(a[0]).chain(once(a[1]))),
                Bytes::Three(a) => self.0.extend(once(a[0]).chain(once(a[1])).chain(once(a[2]))),
            }
        }
    }
}

impl<C> FromIterator<Bytes> for BytesContainer<C>
where C: Default+Extend<u8> {
    fn from_iter<T: IntoIterator<Item = Bytes>>(iter: T) -> Self {
        let mut c = BytesContainer(C::default());
        c.extend(iter);
        c
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

impl<Loc, E> IntoIterator for ResultBytes<Loc, E> {
    type Item = Result<u8, BodyError<Loc, E>>;
    type IntoIter = ResultBytesIter<Loc, E>;
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


impl<Loc, E> Iterator for ResultBytesIter<Loc, E> {
    type Item = Result<u8, BodyError<Loc, E>>;
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

impl<Loc, E> ExactSizeIterator for ResultBytesIter<Loc, E> {}



pub(crate) fn get_6_bits<Location, E>(
    stream: &mut Iterator<Item = Result<(Location, char), E>>,
) -> Result<Option<u8>, BodyError<Location, E>> {
    use self::BodyError::*;

    // Ignore '=' and whitespace
    fn ignore<Location>(c: &(Location, char)) -> bool {
        c.1 != '=' && !is_whitespace(c)
    }
    let mut stream = stream.filter(|c| c.as_ref().map(ignore).unwrap_or(true));

    // If the stream ends without a footer, complain
    let (location, c) = stream.next().ok_or(MissingExpected('-'))??;

    let (offset, base) = match c {
        '-' => return Ok(None),
        'A'...'Z' => (0, 'A'),
        'a'...'z' => (26, 'a'),
        '0'...'9' => (52, '0'),
        '+' => (62, '+'),
        '/' => (63, '/'),
        found => return Err(InvalidCharacter{found, location}),
    };

    Ok(Some((offset + c as u32 - base as u32) as u8))
}


pub struct Chunked<S> {
    stream: S,
    state: Option<ChunkedState>,
}

enum ChunkedState {
    Zero,
    NonZero(ChunkedState2),
}
enum ChunkedState2 {
    One(u8),
    NonOne(ChunkedState3),
}
enum ChunkedState3 {
    Two(u8, u8),
    Three(u8, u8, u8),
}

impl<Loc, E, S> Chunked<S>
where S: Iterator<Item = Result<(Loc, char), E>>
{
    pub fn new(stream: S) -> Self {
        Chunked{
            stream, state: Some(ChunkedState::Zero)
        }
    }
}

impl<Loc, S> Chunked<Map<S, fn((Loc, char)) -> Result<(Loc, char), Void>>>
where S: Iterator<Item = (Loc, char)>
    {
    pub fn from_chars(stream: S) -> Self {
        Self::new(stream.map(map_chars))
    }
}

impl<'a, Loc, E, S> Iterator for Chunked<S>
where Loc: 'a,
    E: 'a,
    S: 'a + Iterator<Item = Result<(Loc, char), E>>
{
    type Item = Result<Bytes, BodyError<Loc, E>>;

    /// May panic if called after it returns `None`
    fn next(&mut self) -> Option<Result<Bytes, BodyError<Loc, E>>> {
        self.state.take().and_then(|state| {
            let (state, result) = state.process(&mut self.stream);
            self.state = state;
            result
        })
    }
}

fn one(a: u8) -> ChunkedState {
    use self::ChunkedState::*;
    use self::ChunkedState2::*;
    NonZero(One(a))
}
fn two(a: u8, b: u8) -> ChunkedState {
    use self::ChunkedState::*;
    use self::ChunkedState2::*;
    use self::ChunkedState3::*;
    NonZero(NonOne(Two(a, b)))
}
fn three(a: u8, b: u8, c: u8) -> ChunkedState {
    use self::ChunkedState::*;
    use self::ChunkedState2::*;
    use self::ChunkedState3::*;
    NonZero(NonOne(Three(a, b, c)))
}

impl ChunkedState {
    fn process<'a, Loc: 'a, E: 'a>(self, stream: &'a mut Iterator<Item = Result<(Loc, char), E>>) -> (Option<Self>, Option<Result<Bytes, BodyError<Loc, E>>>) {
        use self::ChunkedState::*;
        use self::ChunkedState2::*;
        
        let v = match self {
            Zero => match get_6_bits(stream) {
                Err(e) => return (Some(Zero), Some(Err(e))),
                Ok(None) => return (None, None),
                Ok(Some(v)) => One(v << 2),
            },
            NonZero(v) => v,
        };

        v.process(stream)
    }
}

impl ChunkedState2 {
    fn process<'a, Loc: 'a, E: 'a>(self, stream: &'a mut Iterator<Item = Result<(Loc, char), E>>) -> (Option<ChunkedState>, Option<Result<Bytes, BodyError<Loc, E>>>) {
        use self::ChunkedState2::*;
        use self::ChunkedState3::*;
        
        let v = match self {
            One(a) => match get_6_bits(stream) {
                Err(e) => return (Some(one(a)), Some(Err(e))),
                Ok(None) => return (None, Some(Ok(Bytes::One([a])))),
                Ok(Some(v)) => Two(a | (v >> 4), (v & 0b1111) << 4),
            },
            NonOne(v) => v,
        };

        v.process(stream)
    }
}

impl ChunkedState3 {
    fn process<'a, Loc: 'a, E: 'a>(self, stream: &'a mut Iterator<Item = Result<(Loc, char), E>>) -> (Option<ChunkedState>, Option<Result<Bytes, BodyError<Loc, E>>>) {
        use self::ChunkedState::*;
        use self::ChunkedState3::*;
        
        let (a, b, c) = match self {
            Two(a, b) => match get_6_bits(stream) {
                Err(e) => return (Some(two(a, b)), Some(Err(e))),
                Ok(None) => return (None, Some(Ok(Bytes::Two([a, b])))),
                Ok(Some(v)) => (a, b | (v >> 2), (v & 0b11) << 6),
            },
            Three(a, b, c) => (a, b, c),
        };

        match get_6_bits(stream) {
            Err(e) => (Some(three(a, b, c)), Some(Err(e))),
            Ok(None) => (None, Some(Ok(Bytes::Three([a, b, c])))),
            Ok(Some(v)) => (Some(Zero), Some(Ok(Bytes::Three([a, b, c | v])))),
        }
    }
}

pub struct Single<S> {
    stream: S,
    state: Option<SingleState>,
}

enum SingleState {
    ZeroBits,
    NonZeroBits(SingleState2),
}

enum SingleState2 {
    SixBits(u8),
    FourBits(u8),
    TwoBits(u8),
}

impl<Loc, E, S> Single<S>
where S: Iterator<Item = Result<(Loc, char), E>>
{
    pub fn new(stream: S) -> Self {
        Single{
            stream, state: Some(SingleState::ZeroBits)
        }
    }
}

impl<Loc, S> Single<Map<S, fn((Loc, char)) -> Result<(Loc, char), Void>>>
where S: Iterator<Item = (Loc, char)>
    {
    pub fn from_chars(stream: S) -> Self {
        Self::new(stream.map(map_chars))
    }
}

impl<'a, Loc, E, S> Iterator for Single<S>
where Loc: 'a,
    E: 'a,
    S: 'a + Iterator<Item = Result<(Loc, char), E>>
{
    type Item = Result<u8, BodyError<Loc, E>>;

    /// May panic if called after it returns `None`
    fn next(&mut self) -> Option<Result<u8, BodyError<Loc, E>>> {
        self.state.take().and_then(|state| {
            let (state, result) = state.process(&mut self.stream);
            self.state = state;
            result
        })
    }
}

impl SingleState {
    fn process<'a, Loc: 'a, E: 'a>(self, stream: &'a mut Iterator<Item = Result<(Loc, char), E>>) -> (Option<SingleState>, Option<Result<u8, BodyError<Loc, E>>>) {
        use self::SingleState::*;
        use self::SingleState2::*;
        
        let v = if let NonZeroBits(v) = self {
            v
        } else {
            match get_6_bits(stream) {
                Err(e) => return (Some(ZeroBits), Some(Err(e))),
                Ok(None) => return (None, None),
                Ok(Some(v)) => SixBits(v << 2),
            }
        };

        let new = match get_6_bits(stream) {
            Err(e) => return (Some(NonZeroBits(v)), Some(Err(e))),
            Ok(None) => return (None, None),
            Ok(Some(v)) => v,
        };

        match v {
            SixBits(old) => (Some(NonZeroBits(FourBits((new & 0b1111) << 4))), Some(Ok(old | (new >> 4)))),
            FourBits(old) => (Some(NonZeroBits(TwoBits((new & 0b11) << 6))), Some(Ok(old | (new >> 2)))),
            TwoBits(old) => (Some(ZeroBits), Some(Ok(old | new))),
        }
    }
}
