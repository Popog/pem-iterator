use {ParseError, Label};

#[cfg(all(feature = "store_label", not(feature = "std")))]
use MAX_LABEL_SIZE;

pub struct SourceError<E>(pub E);

#[cfg(all(feature = "store_label", not(feature = "std")))]
#[derive(Debug, PartialEq)]
pub struct LabelError {
    pub label: Label,
    pub location: usize,
    pub overflow: char,
}

#[cfg(any(not(feature = "store_label"), feature = "std"))]
pub use Void as LabelError;

pub enum ExpectedError {
    MissingExpected(char),
    Mismatch {
        location: usize,
        expected: char,
        found: char,
    },
}

#[cfg(not(feature = "store_label"))]
pub struct LabelCharacters;

#[cfg(feature = "store_label")]
pub type LabelCharacters = usize;

#[cfg(not(feature = "store_label"))]
impl From<LabelCharacters> for usize {
    fn from(_: LabelCharacters) -> usize {
        0
    }
}

pub fn expect<E>(
    expected: &mut Iterator<Item = char>,
    stream: &mut Iterator<Item = (usize, Result<char, SourceError<E>>)>,
) -> Result<Result<(), SourceError<E>>, ExpectedError> {
    use self::ExpectedError::*;

    for expected in expected {
        let (location, found) = stream.next().ok_or(MissingExpected(expected))?;
        match found {
            Err(e) => return Ok(Err(e)),
            Ok(found) => {
                if expected != found {
                    return Err(Mismatch {
                        expected,
                        found,
                        location,
                    });
                }
            }
        }
    }
    Ok(Ok(()))
}

pub fn expect_begin<E>(
    stream: &mut Iterator<Item = (usize, Result<char, SourceError<E>>)>,
) -> Result<Result<(), SourceError<E>>, ExpectedError> {
    // eat all the whitespace and the BEGINning of the header
    expect(
        &mut "-----BEGIN ".chars(),
        &mut stream.skip_while(|c| c.1.as_ref().ok().map_or(false, is_whitespace)),
    )
}

#[cfg(all(feature = "store_label", not(feature = "std")))]
pub fn expect_label<E>(
    stream: &mut Iterator<Item = (usize, Result<char, SourceError<E>>)>,
) -> Result<Result<(Label, LabelCharacters), SourceError<E>>, LabelError> {
    let mut prev_dash = false;
    let mut label = Label {
        content: [0; MAX_LABEL_SIZE],
        len: 0,
    };
    let mut characters = 0;
    for (location, c) in stream {
        let c = match c {
            Ok(c) => c,
            Err(e) => return Ok(Err(e)),
        };

        // Check for double '-'
        if c == '-' {
            if !prev_dash {
                prev_dash = true;
                continue;
            }
            return Ok(Ok((label, characters)));
        }

        // Add back in any single '-' we skipped over
        if prev_dash {
            if !label.add('-') {
                let location = location - 1;
                return Err(LabelError {
                    label,
                    location,
                    overflow: '-',
                });
            }
            characters += 1;
        }

        if !label.add(c) {
            return Err(LabelError {
                label,
                location,
                overflow: c,
            });
        }
        characters += 1;
    }

    Ok(Ok((label, characters)))
}

#[cfg(all(feature = "store_label", feature = "std"))]
pub fn expect_label<E>(
    stream: &mut Iterator<Item = (usize, Result<char, SourceError<E>>)>,
) -> Result<Result<(Label, LabelCharacters), SourceError<E>>, LabelError> {
    use core::mem::replace;
    let mut prev_dash = false;
    let mut characters = 0;
    let mut v: Result<Label, _> = stream
        .take_while(|c| {
            c.1.as_ref().ok().map_or(true, |c| if !replace(
                &mut prev_dash,
                *c == '-',
            ) || !prev_dash
            {
                characters += 1;
                true
            } else {
                false
            })
        })
        .map(|(_, c)| c)
        .collect();
    // Remove the trailing '-'
    if let Ok(v) = v.as_mut() {
        v.pop();
    }
    Ok(v.map(|v| (v, characters)))
}


#[cfg(not(feature = "store_label"))]
pub fn expect_label<E>(
    stream: &mut Iterator<Item = (usize, Result<char, SourceError<E>>)>,
) -> Result<Result<(Label, LabelCharacters), SourceError<E>>, LabelError> {
    use core::mem::replace;
    let mut prev_dash = true;
    Ok(
        stream
            .take_while(|c| {
                c.1.as_ref().ok().map_or(true, |c| {
                    !replace(&mut prev_dash, *c == '-') || !prev_dash
                })
            })
            .filter_map(|(_, c)| c.err().map(Err))
            .next()
            .unwrap_or(Ok(((), LabelCharacters))),
    )
}

pub fn get_6_bits<E>(
    stream: &mut Iterator<Item = (usize, Result<char, SourceError<E>>)>,
) -> Result<Result<Option<u8>, SourceError<E>>, ParseError> {
    use self::ParseError::*;

    // Ignore '=' and whitespace
    fn ignore(c: &char) -> bool {
        *c != '=' && !is_whitespace(c)
    }
    let mut stream = stream.filter(|c| c.1.as_ref().ok().map_or(true, ignore));

    // If the stream ends without a footer, complain
    let (_, c) = stream.next().ok_or(MissingExpected('-'))?;

    let c = match c {
        Ok(c) => c,
        Err(e) => return Ok(Err(e)),
    };

    let (offset, base) = match c {
        '-' => return Ok(Ok(None)),
        'A'...'Z' => (0, 'A'),
        'a'...'z' => (26, 'a'),
        '0'...'9' => (52, '0'),
        '+' => (62, '+'),
        '/' => (63, '/'),
        _ => return Err(InvalidCharacter(c)),
    };

    Ok(Ok(Some((offset + c as u32 - base as u32) as u8)))
}

#[cfg(feature = "std")]
fn is_whitespace(c: &char) -> bool {
    c.is_whitespace()
}

#[cfg(not(feature = "std"))]
fn is_whitespace(c: &char) -> bool {
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
