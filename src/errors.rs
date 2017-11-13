use parse::{ExpectedError, SourceError, LabelError};

/// Will be replaced with never_type `!` (rust/rust-lang#35121)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Void {}

#[derive(Debug, PartialEq)]
pub enum PreEncapsulationBoundaryError<E> {
    MissingExpected(char),
    Mismatch {
        location: usize,
        expected: char,
        found: char,
    },
    LabelError(LabelError),
    SourceError(E),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissingExpected(char),
    Mismatch {
        location: usize,
        expected: char,
        found: char,
    },
    LabelError(LabelError),
    InvalidCharacter(char),
}

#[derive(Debug, PartialEq)]
pub enum PemError<E> {
    ParseError(ParseError),
    SourceError(E),
}


impl<E> From<SourceError<E>> for PreEncapsulationBoundaryError<E> {
    fn from(e: SourceError<E>) -> Self {
        PreEncapsulationBoundaryError::SourceError(e.0)
    }
}

impl<E> From<ExpectedError> for PreEncapsulationBoundaryError<E> {
    fn from(e: ExpectedError) -> Self {
        use self::PreEncapsulationBoundaryError::*;
        match e {
            ExpectedError::MissingExpected(c) => MissingExpected(c),
            ExpectedError::Mismatch {
                expected,
                found,
                location,
            } => Mismatch {
                expected,
                found,
                location,
            },
        }
    }
}

impl<E> From<LabelError> for PreEncapsulationBoundaryError<E> {
    fn from(e: LabelError) -> Self {
        PreEncapsulationBoundaryError::LabelError(e)
    }
}

impl From<ExpectedError> for ParseError {
    fn from(e: ExpectedError) -> Self {
        match e {
            ExpectedError::MissingExpected(c) => ParseError::MissingExpected(c),
            ExpectedError::Mismatch {
                expected,
                found,
                location,
            } => ParseError::Mismatch {
                expected,
                found,
                location,
            },
        }
    }
}

impl From<LabelError> for ParseError {
    fn from(e: LabelError) -> Self {
        ParseError::LabelError(e)
    }
}

impl From<PreEncapsulationBoundaryError<Void>> for ParseError {
    fn from(e: PreEncapsulationBoundaryError<Void>) -> Self {
        use self::PreEncapsulationBoundaryError::*;
        match e {
            MissingExpected(c) => ParseError::MissingExpected(c),
            Mismatch {
                location,
                expected,
                found,
            } => ParseError::Mismatch {
                location,
                expected,
                found,
            },
            LabelError(label) => ParseError::LabelError(label),
            SourceError(_) => unreachable!(),
        }
    }
}

impl From<PemError<Void>> for ParseError {
    fn from(e: PemError<Void>) -> Self {
        match e {
            PemError::ParseError(e) => e,
            PemError::SourceError(_) => unreachable!(),
        }
    }
}

impl<E> From<SourceError<E>> for PemError<E> {
    fn from(e: SourceError<E>) -> Self {
        PemError::SourceError(e.0)
    }
}

impl<E> From<ExpectedError> for PemError<E> {
    fn from(e: ExpectedError) -> Self {
        PemError::ParseError(e.into())
    }
}

impl<E> From<LabelError> for PemError<E> {
    fn from(e: LabelError) -> Self {
        ParseError::LabelError(e).into()
    }
}

impl<E> From<ParseError> for PemError<E> {
    fn from(e: ParseError) -> Self {
        PemError::ParseError(e)
    }
}

impl<E> From<PreEncapsulationBoundaryError<E>> for PemError<E> {
    fn from(e: PreEncapsulationBoundaryError<E>) -> Self {
        use self::PreEncapsulationBoundaryError::*;
        match e {
            MissingExpected(c) => PemError::ParseError(ParseError::MissingExpected(c)),
            Mismatch {
                location,
                expected,
                found,
            } => PemError::ParseError(ParseError::Mismatch {
                location,
                expected,
                found,
            }),
            LabelError(label) => PemError::ParseError(ParseError::LabelError(label)),
            SourceError(e) => PemError::SourceError(e),
        }
    }
}
