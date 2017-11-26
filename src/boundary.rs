use core::str::Chars;
use core::iter::{Map, once};

use {Void, map_chars, is_whitespace};

#[derive(Debug, PartialEq)]
pub enum EncapsulationBoundaryError<Location, LabelError> {
    MissingExpected(char),
    Mismatch {
        location: Location,
        expected: char,
        found: char,
    },
    LabelError{
        location: Location,
        error: LabelError
    },
}

/// Which boundary to process
pub enum BoundaryType {
    Begin,
    End,
}
/// A trait for extracting the label from a boundary
pub trait Label {
    /// The type of any errors which might occur while accumulating the label
    type LabelError;

    /// Adds a character to the label. If it returns an error, the parsing process will terminate.
    ///
    /// A return value of `Ok(Some(found))` means the label was expected to end.
    /// A return value of `Ok(Some(expected))` means a Mismatch error occured.
    fn push(&mut self, found: char) -> Result<Option<char>, Self::LabelError>;

    /// Signals that the label is complete because a double `'-'` was reached.
    ///
    /// Defaults to `Ok(None)`.
    /// A return value of `Ok(Some(expected))` is interpreted as a Mismatch error.
    /// A return value of `Ok(Some('-'))` is invalid and reserved for future use.
    fn complete(&mut self) -> Result<Option<char>, Self::LabelError> {
        Ok(None)
    }
}

/// A label "accumulator" which discards the label
pub struct DiscardLabel;

/// A label accumulator which calls the specified function
pub struct LabelFn<F>(pub F);

/// A label accumulator which matches against existing characters
pub struct LabelMatcher<I>(pub I);

impl<'a, T: 'a+Extend<char>> Label for &'a mut T {
    type LabelError = Void;
    fn push(&mut self, found: char) -> Result<Option<char>, Self::LabelError> {
        self.extend(once(found));
        Ok(None)
    }
}

impl Label for DiscardLabel {
    type LabelError = Void;
    fn push(&mut self, _: char) -> Result<Option<char>, Self::LabelError> {
        Ok(None)
    }
}

impl <T, F> Label for LabelFn<F>
where F: FnMut(char) -> Result<Option<char>, T> {
    type LabelError = T;
    fn push(&mut self, found: char) -> Result<Option<char>, T> {
        self.0(found)
    }
}

impl<T: Iterator<Item=char>> Label for LabelMatcher<T> {
    type LabelError = Void;
    fn push(&mut self, found: char) -> Result<Option<char>, Self::LabelError> {
        Ok(match self.0.next() {
            Some(expected) => if expected == found {
                None
            } else {
                Some(expected)
            },
            None => Some(found),
        })
    }

    fn complete(&mut self) -> Result<Option<char>, Self::LabelError> {
        Ok(self.0.next())
    }
}



pub struct BoundaryParser<Loc, Lbl: Label, S> {
    stream: S,
    state: Option<BoundaryParserState<Loc, Lbl>>,
    result: Result<(),EncapsulationBoundaryError<Loc, Lbl::LabelError>>,
}

enum BoundaryParserState<Loc, Lbl> {
    EatFirst{
        label: Lbl,
        b: BoundaryType,
    },
    NotEatFirst(BoundaryParserState2<Loc, Lbl>),
}

enum BoundaryParserState2<Loc, Lbl> {
    EatKey{
        label: Lbl,
        key: Chars<'static>,
        expected: char,
    },
    NotEatKey(BoundaryParserState3<Loc, Lbl>)
}

enum BoundaryParserState3<Loc, Lbl> {
    EatLabel{
        label: Lbl,
        prev_dash: Option<Loc>,
    },
    EatEnd{
        end: Chars<'static>,
        expected: char,
    },
}

impl<Loc, Lbl, E, S> BoundaryParser<Loc, Lbl, S>
where Lbl: Label,
    S: Iterator<Item = Result<(Loc, char), E>>
    {
    pub fn new(b: BoundaryType, stream: S, label: Lbl) -> Self {
        BoundaryParser{
            stream, state: Some(BoundaryParserState::EatFirst{label, b}), result: Ok(()),
        }
    }

    /// Call after `next` returns None
    pub fn complete(self) -> Result<(), EncapsulationBoundaryError<Loc, Lbl::LabelError>> {
        self.result
    }
}

impl<Loc, Lbl, S> BoundaryParser<Loc, Lbl, Map<S, fn((Loc, char)) -> Result<(Loc, char), Void>>>
where Lbl: Label,
    S: Iterator<Item = (Loc, char)>
    {
    pub fn from_chars(b: BoundaryType, stream: S, label: Lbl) -> Self {
        Self::new(b, stream.map(map_chars), label)
    }
}

impl<Loc, Lbl, E, S> Iterator for BoundaryParser<Loc, Lbl, S>
where Lbl: Label,
    S: Iterator<Item = Result<(Loc, char), E>>
{
    type Item = E;
    /// Panics if called after it returns `None`
    fn next(&mut self) -> Option<E> {
        match self.state.take().unwrap().process(&mut self.stream) {
            Err(e) => {
                self.result = Err(e);
                None
            },
            Ok(None) => None,
            Ok(Some((s, e))) => {
                self.state = Some(s);
                Some(e)
            },
        }
    } 
}

impl<Loc, Lbl: Label> BoundaryParserState<Loc, Lbl> {
    fn process<'a, E: 'a>(self, stream: &'a mut Iterator<Item = Result<(Loc, char), E>>) -> Result<Option<(Self, E)>, EncapsulationBoundaryError<Loc, Lbl::LabelError>> {
        use self::EncapsulationBoundaryError::*;
        use self::BoundaryParserState::*;
        use self::BoundaryParserState2::*;
        

        let v = match self {
            EatFirst{label, b} => {
                // For BEGIN, eat all the whitespace and the first '-'
                // END has already had one '-' eaten during body parsing, so don't worry about that
                let key = match b {
                    BoundaryType::Begin => {
                        match stream.skip_while(|c| c.as_ref().ok().map_or(false, is_whitespace)).next() {
                            Some(Err(e)) => return Ok(Some((EatFirst{label, b}, e))),
                            None => return Err(MissingExpected('-')),
                            Some(Ok((location, found))) => if found != '-' {
                                return Err(Mismatch{found, location, expected: '-'})
                            },
                        }
                        "---BEGIN "
                    },
                    BoundaryType::End => "---END ",
                }.chars();

                EatKey{label, key, expected: '-'}
            },
            NotEatFirst(v) => v,
        };

        v.process(stream).map(|v| v.map(|(v, e)| (NotEatFirst(v), e)))
    }
}

impl<Loc, Lbl: Label> BoundaryParserState2<Loc, Lbl> {
    fn process<'a, E: 'a>(self, stream: &'a mut Iterator<Item = Result<(Loc, char), E>>) -> Result<Option<(Self, E)>, EncapsulationBoundaryError<Loc, Lbl::LabelError>> {
        use self::EncapsulationBoundaryError::*;
        use self::BoundaryParserState2::*;
        use self::BoundaryParserState3::*;

        let v = match self {
            EatKey{label, mut key, mut expected} => loop {
                match stream.next() {
                    Some(Err(e)) => return Ok(Some((EatKey{label, key, expected}, e))),
                    None => return Err(MissingExpected(expected)),
                    Some(Ok((location, found))) => if found != expected {
                        return Err(Mismatch{found, location, expected})
                    } else if let Some(e) = key.next() {
                        expected = e;
                    } else {
                        break EatLabel{label, prev_dash: None}
                    },
                }
            },
            NotEatKey(v) => v,
        };

        v.process(stream).map(|v| v.map(|(v, e)| (NotEatKey(v), e)))
    }
}

impl<Loc, Lbl: Label> BoundaryParserState3<Loc, Lbl> {
    fn process<'a, E: 'a>(self, stream: &'a mut Iterator<Item = Result<(Loc, char), E>>) -> Result<Option<(Self, E)>, EncapsulationBoundaryError<Loc, Lbl::LabelError>> {
        use self::EncapsulationBoundaryError::*;
        use self::BoundaryParserState3::*;

        let (mut end, mut expected) = match self {
            EatLabel{mut label, mut prev_dash} => loop {
                use self::EncapsulationBoundaryError::*;

                let v = stream.next();
                let (location, c) = match v {
                    Some(Err(e)) => return Ok(Some((EatLabel{label, prev_dash}, e))),
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

                    // Done, find the last 3 dashes
                    break ("--".chars(), '-');
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
            },
            EatEnd{end, expected} => (end, expected),
        };

        loop {
            match stream.next() {
                Some(Err(e)) => return Ok(Some((EatEnd{end, expected}, e))),
                None => return Err(MissingExpected(expected)),
                Some(Ok((location, found))) => if found != expected {
                    return Err(Mismatch{found, location, expected: expected})
                } else if let Some(e) = end.next() {
                    expected = e;
                } else {
                    return Ok(None)
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundaryType, BoundaryParser, LabelMatcher};
    #[cfg(not(feature = "std"))]
    use super::DiscardLabel;

    #[test]
    fn test_parse_boundary() {
        #[cfg(feature = "std")]
        fn helper(b: BoundaryType, input: &str, _label: &str) {
            #[cfg(feature = "std")]
            let mut label_buf = String::new();

            {
                let mut parser = BoundaryParser::from_chars(b, input.chars().enumerate(), &mut label_buf);
                assert_eq!(parser.next(), None);
                assert_eq!(parser.complete(), Ok(()));
            }

            assert_eq!(_label, label_buf.as_str());
        }
        #[cfg(not(feature = "std"))]
        fn helper(b: BoundaryType, input: &str, _label: &str) {
            let mut parser = BoundaryParser::from_chars(b, input.chars().enumerate(), DiscardLabel);
            assert_eq!(parser.next(), None);
            assert_eq!(parser.complete(), Ok(()));
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
            let mut verifier = BoundaryParser::from_chars(b, input.chars().enumerate(), LabelMatcher(label.chars()));
            assert_eq!(verifier.next(), None);
            assert_eq!(verifier.complete(), Ok(()));
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
