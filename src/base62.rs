pub use self::DecodeError::*;
use std::fmt;

const BASE: u128 = 62;
const MAX_DECODE_LEN: usize = 111;
const ALPHABET: [u8; BASE as usize] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
    b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V',
    b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l',
    b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
];

pub enum DecodeError {
    InvalidBase62Byte(char, usize),
    ArithmeticOverflow,
}

impl fmt::Debug for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidBase62Byte(ch, idx) => {
                write!(f, "Invalid character '{}' at position {}", ch, idx)
            }
            ArithmeticOverflow => write!(f, "Decode result is too large"),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[must_use]
pub fn encode<T: Into<u128>>(num: T) -> String {
    let mut num = num.into();
    if num == 0 {
        return "0".to_owned();
    }
    let mut bytes: [u8; MAX_DECODE_LEN] = [0; MAX_DECODE_LEN];

    let mut i: usize = 0;
    loop {
        if num == 0 {
            break;
        }
        i += 1;

        bytes[MAX_DECODE_LEN - i] = ALPHABET[(num % BASE) as usize];
        num /= BASE;
    }

    String::from_utf8(bytes[MAX_DECODE_LEN - i..MAX_DECODE_LEN].to_vec()).unwrap()
}

pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<u128, DecodeError> {
    let mut result = 0;
    let input = input.as_ref();

    for (i, c) in input.iter().rev().enumerate() {
        let num = BASE.checked_pow(i as u32).ok_or(ArithmeticOverflow)?;
        match ALPHABET.binary_search(&c) {
            Ok(v) => match (v as u128).checked_mul(num) {
                Some(z) => result += z,
                None => return Err(ArithmeticOverflow),
            },
            Err(_e) => {
                return Err(InvalidBase62Byte(c.to_owned() as char, input.len() - i));
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::base62;
    use quickcheck::TestResult;

    quickcheck! {
        fn encode_decode(xs: u128) -> bool {
            let encoded = base62::encode(xs);
            xs == base62::decode(&encoded).unwrap()
        }
    }

    quickcheck! {
        fn decode_good(xs: String) -> TestResult {
            if xs.chars().all(|c| c.is_ascii() && base62::ALPHABET.binary_search(&(c as u8)).is_ok()) && !xs.is_empty() {
                return TestResult::from_bool(base62::decode(&xs).is_ok());
            } else {
                TestResult::discard()
            }
        }
    }

    quickcheck! {
        fn decode_bad(xs: String) -> TestResult {
            if xs.chars().all(|c| c.is_ascii() && base62::ALPHABET.binary_search(&(c as u8)).is_ok()) {
                TestResult::discard()
            } else {
                return TestResult::from_bool(base62::decode(&xs).is_err());
            }
        }
    }

    #[test]
    fn test_encode() {
        assert_eq!(base62::encode(u128::MAX), "7n42DGM5Tflk9n8mt7Fhc7");
        assert_eq!(base62::encode(0u8), "0");
    }

    #[test]
    fn test_decode() {
        assert_eq!(base62::decode("7n42DGM5Tflk9n8mt7Fhc7").unwrap(), u128::MAX);
        assert_eq!(base62::decode("").unwrap(), 0);
    }

    #[test]
    fn test_decode_invalid_char() {
        assert!(base62::decode("ds{Z455f").is_err());
    }

    #[test]
    fn test_decode_long_string() {
        assert!(base62::decode("7n42DGM5Tflk9n8mt7Fhc7B").is_err());
    }
}
