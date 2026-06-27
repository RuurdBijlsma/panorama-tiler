/// The character set defined by Pannellum for Base83 compression.
const B83_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz#$%*+,-.:;=?@[]^_{|}~";

/// Encodes an array of unsigned 32-bit integers into a Base83 representation.
/// Pad length specifies the character allocation per integer value.
pub fn encode(vals: &[u32], length: usize) -> String {
    let mut result = String::with_capacity(vals.len() * length);
    for &val in vals {
        for i in 1..=length {
            let power = 83u32.pow((length - i) as u32);
            let idx = (val / power) % 83;
            result.push(B83_CHARS[idx as usize] as char);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b83_single_values() {
        assert_eq!(encode(&[0], 1), "0");
        assert_eq!(encode(&[5], 1), "5");
        assert_eq!(encode(&[82], 1), "~");
    }

    #[test]
    fn test_b83_padded_values() {
        // Matches Python: int(100 // 83^1) % 83 -> 1 ('1')
        //                 int(100 // 83^0) % 83 -> 17 ('H')
        assert_eq!(encode(&[100], 2), "1H");
    }
}