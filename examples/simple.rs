
extern crate pem_iterator;

use pem_iterator::ParseError;

const SAMPLE: &'static str = "-----BEGIN RSA PRIVATE KEY-----
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
-----END RSA PRIVATE KEY-----";


#[cfg(not(feature = "std"))]
fn parse_chunked() -> Result<(), ParseError> {
    use pem_iterator::chunked::{Pem, ResultBytes};

    let pem = Pem::from_chars(SAMPLE.chars())?;

    // This is only possible if we have the store_label feature enabled
    #[cfg(feature = "store_label")]
    println!("PEM label: {}", pem.label());

    // Without std, the library doesn't provide the automatic adapter for Vec
    let data: Result<Vec<u8>, _> = pem.flat_map(ResultBytes).collect();
    let data = data?;

    println!("data label: {:?}", data);
    Ok(())
}

#[cfg(feature = "std")]
fn parse_chunked() -> Result<(), ParseError> {
    use pem_iterator::chunked::Pem;

    let pem = Pem::from_chars(SAMPLE.chars())?;

    // This is only possible if we have the store_label feature enabled
    #[cfg(feature = "store_label")]
    println!("PEM label: {}", pem.label());

    // Collect the data
    let data: Result<Vec<u8>, _> = pem.collect();
    let data = data?;

    println!("data label: {:?}", data);
    Ok(())
}

fn parse_single() -> Result<(), ParseError> {
    use pem_iterator::single::Pem;

    let pem = Pem::from_chars(SAMPLE.chars())?;

    // This is only possible if we have the store_label feature enabled
    #[cfg(feature = "store_label")]
    println!("PEM label: {}", pem.label());

    // Single never needs an adaptor
    let data: Result<Vec<u8>, _> = pem.collect();
    let data = data?;

    println!("data label: {:?}", data);
    Ok(())
}

fn main() {
    parse_chunked().unwrap();
    parse_single().unwrap();
}
