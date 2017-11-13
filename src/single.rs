use core::marker::PhantomData;
use core::iter::{Map, once};

use {PreEncapsulationBoundaryError, Label, PemError, ParseError, Void};
use parse::{SourceError, LabelCharacters, expect, expect_begin, expect_label, get_6_bits};

pub struct Pem<E, R> {
    label: Label,
    characters: LabelCharacters,
    state: State,
    _marker: PhantomData<*const E>,
    count: usize,
    stream: R,
}

enum State {
    Normal(NormalState),
    Complete(Option<ParseError>),
}

#[derive(Clone, Copy)]
enum NormalState {
    Base64(Option<Base64Cycle>),
    PostEncapsulationBoundary(PostEncapsulationBoundary),
}

#[derive(Clone, Copy)]
enum Base64Cycle {
    SixBits(u8),
    FourBits(u8),
    TwoBits(u8),
}

#[derive(Clone, Copy)]
enum PostEncapsulationBoundary {
    DashStart(usize),
    Label(usize),
    DashEnd(usize),
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
    /// Create a PEM parser that parses 1 byte of output at at time
    pub fn new(mut stream: R) -> Result<Self, PreEncapsulationBoundaryError<E>> {
        let mut count: usize = 0;
        let (label, characters) = {
            let mut stream = stream.by_ref().map(|a| {
                let i = count;
                count += 1;
                (i, a.map_err(SourceError))
            });

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
            state: State::Normal(NormalState::Base64(None)),
            _marker: PhantomData,
            count,
        })
    }

    /// Process the post-encapsulation boundary
    ///
    /// If we hit a `ParseError`, we can stop immediately, as such errors are unrecoverable
    fn process_peb(
        &mut self,
        mut peb: PostEncapsulationBoundary,
    ) -> Result<Result<(), SourceError<E>>, ParseError> {
        let count = &mut self.count;
        let mut stream = self.stream.by_ref().map(|a| {
            let i = *count;
            *count += 1;
            (i, a.map_err(SourceError))
        });

        loop {
            match peb {
                PostEncapsulationBoundary::DashStart(i) => {
                    const START: &'static str = "----END ";
                    for (i, c) in START.chars().enumerate().skip(i) {
                        if let Err(e) = expect(&mut once(c), &mut stream)? {
                            self.state = State::Normal(NormalState::PostEncapsulationBoundary(
                                PostEncapsulationBoundary::DashStart(i),
                            ));
                            return Ok(Err(e));
                        }
                    }
                    peb = PostEncapsulationBoundary::Label(0);
                }
                PostEncapsulationBoundary::Label(_i) => {
                    #[cfg(feature = "store_label")]
                    {
                        for (i, c) in self.label.as_str().chars().enumerate().skip(_i as usize) {
                            if let Err(e) = expect(&mut once(c), &mut stream)? {
                                self.state = State::Normal(NormalState::PostEncapsulationBoundary(
                                    PostEncapsulationBoundary::Label(i),
                                ));
                                return Ok(Err(e));
                            }
                        }
                    }
                    #[cfg(not(feature = "store_label"))]
                    {
                        if let Err(e) = expect_label(&mut stream)? {
                            return Ok(Err(e.into()));
                        }
                    }

                    peb = PostEncapsulationBoundary::DashEnd(0);
                }
                PostEncapsulationBoundary::DashEnd(i) => {
                    const END: &'static str = "---";
                    for (i, c) in END.chars().enumerate().skip(i) {
                        if let Err(e) = expect(&mut once(c), &mut stream)? {
                            self.state = State::Normal(NormalState::PostEncapsulationBoundary(
                                PostEncapsulationBoundary::DashEnd(i),
                            ));
                            return Ok(Err(e));
                        }
                    }
                    return Ok(Ok(()));
                }
            }
        }
    }

    fn get_6_bits(&mut self) -> Result<Option<u8>, PemError<E>> {
        let count = &mut self.count;
        let mut stream = self.stream.by_ref().map(|a| {
            let i = *count;
            *count += 1;
            (i, a.map_err(SourceError))
        });
        Ok(get_6_bits(&mut stream)??)
    }

    #[cfg(feature = "store_label")]
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
            state,
            count,
            _marker,
        } = self;
        let stream = f(stream);
        Pem {
            label,
            characters,
            stream,
            state,
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
    type Item = Result<u8, PemError<E>>;
    fn next(&mut self) -> Option<Self::Item> {
        use self::State::*;
        use self::Base64Cycle::*;
        use self::PostEncapsulationBoundary::*;
        use self::NormalState::*;

        // Check if we're complete or still processing
        // Return and clear any pending errors if we're complete
        let state = match &mut self.state {
            &mut Normal(state) => state,
            &mut Complete(ref mut complete) => return complete.take().map(PemError::from).map(Err),
        };

        // Check if we're processing the base64 or the post-encapsulation boundary
        let cycle = match state {
            Base64(cycle) => cycle,
            PostEncapsulationBoundary(peb) => {
                return match self.process_peb(peb) {
                    // `ParseError`s are fatal
                    Err(e) => {
                        self.state = State::Complete(None);
                        Some(Err(e.into()))
                    }
                    // `SourceError`s are not
                    Ok(Err(e)) => Some(Err(e.into())),
                    Ok(Ok(())) => None,
                };
            }
        };

        // Get 6 bits
        let new = match self.get_6_bits() {
            Err(e) => return Some(Err(e)),
            Ok(None) => {
                let peb = DashStart(0);

                if let Some(cycle) = cycle {
                    self.state = Normal(PostEncapsulationBoundary(peb));
                    return Some(Ok(match cycle {
                        SixBits(old) => old << 2,
                        FourBits(old) => old << 4,
                        TwoBits(old) => old << 6,
                    }));
                }

                // Try to get the footer
                return match self.process_peb(peb) {
                    Err(e) => {
                        self.state = Complete(None);
                        Some(Err(e.into()))
                    }
                    Ok(Err(e)) => Some(Err(e.into())),
                    Ok(Ok(())) => None,
                };
            }
            Ok(Some(i)) => i,
        };

        // See how that adjusts our cycle
        if let Some(cycle) = cycle {
            let (cycle, result) = match cycle {
                SixBits(old) => (Some(FourBits(new & 0b1111)), (old << 2) | (new >> 4)),
                FourBits(old) => (Some(TwoBits(new & 0b11)), (old << 4) | (new >> 2)),
                TwoBits(old) => (None, (old << 6) | new),
            };
            self.state = Normal(Base64(cycle));
            return Some(Ok(result));
        }

        // If we got here, we had 0 bits, so save the 6 new bits to the cycle and try to get 6 more.
        self.state = Normal(Base64(Some(SixBits(new))));
        let old = new;

        Some(match self.get_6_bits() {
            Err(e) => Err(e),
            Ok(None) => {
                self.state = Normal(PostEncapsulationBoundary(DashStart(0)));
                Ok(old << 2)
            }
            Ok(Some(new)) => {
                self.state = Normal(Base64(Some(FourBits(new & 0b1111))));
                Ok((old << 2) | (new >> 4))
            }
        })
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
