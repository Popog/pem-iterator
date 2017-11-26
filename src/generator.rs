use core::ops::{Generator};

use boundary::{BoundaryType, EncapsulationBoundaryError, Label};
use body::{BodyError, Bytes, get_6_bits};
use {Void, map_chars, is_whitespace};


pub fn parse_boundary_chars<Loc, Lbl, S>(b: BoundaryType, stream: S, label: Lbl) -> impl Generator<
    Yield=Void,
    Return=Result<(), EncapsulationBoundaryError<Loc, Lbl::LabelError>>,
>
where Lbl: Label,
    S: Iterator<Item = (Loc, char)> {
    parse_boundary(b, stream.map(map_chars), label)
}


/// Parses a boundary and extracts the label.
///
/// To save the `label`, pass `&mut String`, to discard, pass `&mut ()`.
pub fn parse_boundary<Loc, Lbl, E, S>(b: BoundaryType, mut stream: S, mut label: Lbl) -> impl Generator<
    Yield=E,
    Return=Result<(), EncapsulationBoundaryError<Loc, Lbl::LabelError>>,
>
where Lbl: Label,
    S: Iterator<Item = Result<(Loc, char), E>> {
    move ||{
        // For BEGIN, eat all the whitespace and the first '-'
        // END has already had one '-' eaten during body parsing, so don't worry about that
        let key = match b {
            BoundaryType::Begin => {
                loop {
                    use self::EncapsulationBoundaryError::*;
                    let v = stream.by_ref()
                        .skip_while(|c| c.as_ref().ok().map_or(false, is_whitespace)).next();
                    match v {
                        Some(Err(e)) => yield e,
                        None => return Err(MissingExpected('-')),
                        Some(Ok((location, found))) => if found != '-' {
                            return Err(Mismatch {expected: '-', found, location})
                        } else {
                            break
                        }
                    }
                }
                "----BEGIN "
            },
            BoundaryType::End => "----END ",
        };

        // Eat the rest of the key
        for expected in key.chars() {
            use self::EncapsulationBoundaryError::*;
            loop {
                let v = stream.next();
                match v {
                    Some(Err(e)) => yield e,
                    None => return Err(MissingExpected(expected)),
                    Some(Ok((location, found))) => if expected != found {
                        return Err(Mismatch {expected, found, location})
                    } else {
                        break
                    }
                }
            }
        }

        // Get the label (this will eat the first two '-'s of the boundary end)
        let mut prev_dash = None;
        loop {
            use self::EncapsulationBoundaryError::*;

            let v = stream.next();
            let (location, c) = match v {
                Some(Err(e)) => {
                    yield e;
                    continue
                },
                None => return Err(MissingExpected('-')),
                Some(Ok(c)) => c,
            };

            // Check for double '-'
            if c == '-' {
                if prev_dash.is_none() {
                    prev_dash = Some(location);
                    continue;
                }

                match label.complete() {
                    Ok(None) => {},
                    Err(error) => return Err(LabelError{error, location}),
                    Ok(Some(expected)) => return Err(Mismatch{location, expected, found: c}),
                }

                break;
            }

            // Add back in any single '-' we skipped over
            if let Some(prev_location) = prev_dash.take() {
                match label.push('-') {
                    Ok(None) => {},
                    Err(error) => return Err(LabelError{error, location: prev_location}),
                    Ok(Some(expected)) => return Err(if expected == '-' {
                        Mismatch{location, expected, found: c}
                    } else {
                        Mismatch{location: prev_location, expected, found: '-'}
                    })
                }
            }

            match label.push(c) {
                Ok(None) => {},
                Err(error) => return Err(LabelError{error, location}),
                Ok(Some(expected)) => return Err(if expected == c {
                    Mismatch{location, expected: '-', found: c}
                } else {
                    Mismatch{location, expected, found: c}
                }),
            }
        }

        // Eat the end of the boundary
        for expected in "---".chars() {
            use self::EncapsulationBoundaryError::*;
            loop {
                let v = stream.next();
                match v {
                    Some(Err(e)) => yield e,
                    None => return Err(MissingExpected(expected)),
                    Some(Ok((location, found))) => if expected != found {
                        return Err(Mismatch {expected, found, location})
                    } else {
                        break
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn parse_body_chunked_chars<Location, S>(stream: S) -> impl Generator<
    Yield=Result<Bytes, BodyError<Location, Void>>,
    Return=(),
>
where S: Iterator<Item = (Location, char)> {
    parse_body_chunked(stream.map(map_chars))
}

/// Parses the body in chunks.
///
/// Stops after consuming a single `-`.
pub fn parse_body_chunked<Location, E, S>(mut stream: S) -> impl Generator<
    Yield=Result<Bytes, BodyError<Location, E>>,
    Return=(),
>
where S: Iterator<Item = Result<(Location, char), E>> {
    move ||{
        loop {
            let a = loop {
                let v = get_6_bits(&mut stream);
                match v {
                    Err(e) => yield Err(e),
                    Ok(None) => return,
                    Ok(Some(v)) => break v << 2,
                }
            };

            let (a, b) = loop {
                let v = get_6_bits(&mut stream);
                match v {
                    Err(e) => yield Err(e),
                    Ok(None) => {
                        yield Ok(Bytes::One([a]));
                        return
                    },
                    Ok(Some(v)) => break (a | (v >> 4), (v & 0b1111) << 4),
                }
            };

            let (b, c) = loop {
                let v = get_6_bits(&mut stream);
                match v {
                    Err(e) => yield Err(e),
                    Ok(None) => {
                        yield Ok(Bytes::Two([a, b]));
                        return
                    },
                    Ok(Some(v)) => break (b | (v >> 2), (v & 0b11) << 6),
                }
            };

            let c = loop {
                let v = get_6_bits(&mut stream);
                match v {
                    Err(e) => yield Err(e),
                    Ok(None) => {
                        yield Ok(Bytes::Three([a, b, c]));
                        return
                    },
                    Ok(Some(v)) => break c | v,
                }
            };

            yield Ok(Bytes::Three([a, b, c]));
        }
    }
}

pub fn parse_body_single_chars<Location, S>(stream: S) -> impl Generator<
    Yield=Result<u8, BodyError<Location, Void>>,
    Return=(),
>
where S: Iterator<Item = (Location, char)> {
    parse_body_single(stream.map(map_chars))
}

/// Parses the body one byte of output at a time.
///
/// Stops after consuming a single `-`.
pub fn parse_body_single<Location, E, S>(mut stream: S) -> impl Generator<
    Yield=Result<u8, BodyError<Location, E>>,
    Return=(),
>
where S: Iterator<Item = Result<(Location, char), E>> {
    move || {
        loop {
            let o = loop {
                let v = get_6_bits(&mut stream);
                match v {
                    Err(e) => yield Err(e),
                    Ok(None) => return,
                    Ok(Some(v)) => break v << 2,
                }
            };

            let o = loop {
                let v = get_6_bits(&mut stream);
                match v {
                    Err(e) => yield Err(e),
                    Ok(None) => {
                        yield Ok(o);
                        return
                    },
                    Ok(Some(v)) => {
                        yield Ok(o | (v >> 4));
                        break (v & 0b1111) << 4;
                    },
                }
            };

            let o = loop {
                let v = get_6_bits(&mut stream);
                match v {
                    Err(e) => yield Err(e),
                    Ok(None) => {
                        yield Ok(o);
                        return
                    },
                    Ok(Some(v)) => {
                        yield Ok(o | (v >> 2));
                        break (v & 0b11) << 6;
                    },
                }
            };

            loop {
                let v = get_6_bits(&mut stream);
                match v {
                    Err(e) => yield Err(e),
                    Ok(None) => {
                        yield Ok(o);
                        return
                    },
                    Ok(Some(v)) =>{
                        yield Ok(o | v);
                        break;
                    },
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use core::ops::{Generator, GeneratorState};
    use boundary::{BoundaryType, LabelMatcher};
    #[cfg(not(feature = "std"))]
    use boundary::DiscardLabel;
    use super::{parse_boundary_chars};


    #[test]
    fn test_parse_boundary() {
        fn helper(b: BoundaryType, input: &str, _label: &str) {
            #[cfg(feature = "std")]
            let mut label_buf = String::new();

            {
                #[cfg(not(feature = "std"))]
                let label_buf = DiscardLabel;
                #[cfg(feature = "std")]
                let label_buf = &mut label_buf;

                let mut gen = parse_boundary_chars(b, input.chars().enumerate(), label_buf);
                match gen.resume() {
                    GeneratorState::Yielded(_) => unreachable!(),
                    GeneratorState::Complete(r) => assert_eq!(r, Ok(())),
                }
            }

            #[cfg(feature = "std")]
            assert_eq!(_label, label_buf.as_str());
        }

        const BEGIN_PRIVATE: &'static str = "-----BEGIN RSA PRIVATE KEY-----";
        const BEGIN_INT_CERT: &'static str = "-----BEGIN INTERMEDIATE CERT-----";
        const BEGIN_CERT: &'static str = "-----BEGIN CERTIFICATE-----";
        const BEGIN_COMPLEX: &'static str = "\t\r -----BEGIN \u{211D}-\u{212D}-----";

        // END has one fewer '-' because body parsing consumes the first one
        const END_PRIVATE: &'static str = "----END RSA PRIVATE KEY-----";
        const END_INT_CERT: &'static str = "----END INTERMEDIATE CERT-----";
        const END_CERT: &'static str = "----END CERTIFICATE-----";
        const END_COMPLEX: &'static str = "----END \u{211D}-\u{212D}-----";

        helper(BoundaryType::Begin, BEGIN_CERT, "CERTIFICATE");
        helper(BoundaryType::Begin, BEGIN_INT_CERT, "INTERMEDIATE CERT");
        helper(BoundaryType::Begin, BEGIN_PRIVATE, "RSA PRIVATE KEY");
        helper(BoundaryType::Begin, BEGIN_COMPLEX, "\u{211D}-\u{212D}");

        helper(BoundaryType::End, END_CERT, "CERTIFICATE");
        helper(BoundaryType::End, END_INT_CERT, "INTERMEDIATE CERT");
        helper(BoundaryType::End, END_PRIVATE, "RSA PRIVATE KEY");
        helper(BoundaryType::End, END_COMPLEX, "\u{211D}-\u{212D}");
    }
    
    #[test]
    fn test_verify_boundary() {
        fn helper(b: BoundaryType, input: &str, label: &str) {
            let mut gen = parse_boundary_chars(b, input.chars().enumerate(), LabelMatcher(label.chars()));
            
            match gen.resume() {
                GeneratorState::Yielded(_) => unreachable!(),
                GeneratorState::Complete(r) => assert_eq!(r, Ok(())),
            }
        }

        const BEGIN_PRIVATE: &'static str = "-----BEGIN RSA PRIVATE KEY-----";
        const BEGIN_INT_CERT: &'static str = "-----BEGIN INTERMEDIATE CERT-----";
        const BEGIN_CERT: &'static str = "-----BEGIN CERTIFICATE-----";
        const BEGIN_COMPLEX: &'static str = "\t\r -----BEGIN \u{211D}-\u{212D}-----";

        // END has one fewer '-' because body parsing consumes the first one
        const END_PRIVATE: &'static str = "----END RSA PRIVATE KEY-----";
        const END_INT_CERT: &'static str = "----END INTERMEDIATE CERT-----";
        const END_CERT: &'static str = "----END CERTIFICATE-----";
        const END_COMPLEX: &'static str = "----END \u{211D}-\u{212D}-----";

        helper(BoundaryType::Begin, BEGIN_CERT, "CERTIFICATE");
        helper(BoundaryType::Begin, BEGIN_INT_CERT, "INTERMEDIATE CERT");
        helper(BoundaryType::Begin, BEGIN_PRIVATE, "RSA PRIVATE KEY");
        helper(BoundaryType::Begin, BEGIN_COMPLEX, "\u{211D}-\u{212D}");

        helper(BoundaryType::End, END_CERT, "CERTIFICATE");
        helper(BoundaryType::End, END_INT_CERT, "INTERMEDIATE CERT");
        helper(BoundaryType::End, END_PRIVATE, "RSA PRIVATE KEY");
        helper(BoundaryType::End, END_COMPLEX, "\u{211D}-\u{212D}");
    }
}
