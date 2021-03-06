
extern crate pem_iterator;

use pem_iterator::boundary::{BoundaryType, BoundaryParser, DiscardLabel};
use pem_iterator::body::Chunked;

const SAMPLE: &'static str = "-----BEGIN RSA PRIVATE KEY-----
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
-----END RSA PRIVATE KEY-----";

fn main() {
    let mut input = SAMPLE.chars().enumerate();

    // In this example, we don't care what the label is
    {
        let mut parser = BoundaryParser::from_chars(BoundaryType::Begin, &mut input, DiscardLabel);
        assert_eq!(parser.next(), None);
        assert_eq!(parser.complete(), Ok(()));
    }

    // Parse the body
    let data: Result<Vec<u8>, _> = Chunked::from_chars(&mut input).collect();
    let data = data.unwrap();

    // We also don't care what the label in the end boundary is. They don't even have to match
    {
        let mut parser = BoundaryParser::from_chars(BoundaryType::End, &mut input, DiscardLabel);
        assert_eq!(parser.next(), None);
        assert_eq!(parser.complete(), Ok(()));
    }

    println!("data: {:?}", data);
}
