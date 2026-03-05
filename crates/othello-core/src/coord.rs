pub fn col_to_letter(col: usize) -> char {
    (b'a' + col as u8) as char
}

pub fn letter_to_col(c: char) -> Option<usize> {
    let c = c.to_ascii_lowercase();
    if c.is_ascii_lowercase() && c <= 't' {
        Some((c as u8 - b'a') as usize)
    } else {
        None
    }
}

pub fn to_gtp(row: usize, col: usize) -> String {
    format!("{}{}", col_to_letter(col), row + 1)
}

pub fn from_gtp(s: &str, size: usize) -> Result<(usize, usize), String> {
    let s = s.trim().to_lowercase();
    if s == "pass" {
        return Err("pass".to_string());
    }
    let mut chars = s.chars();
    let col_char = chars.next().ok_or_else(|| "Empty coordinate".to_string())?;
    let col = letter_to_col(col_char).ok_or_else(|| format!("Invalid column: {}", col_char))?;
    let row_str: String = chars.collect();
    let row_num: usize = row_str
        .parse()
        .map_err(|_| format!("Invalid row: {}", row_str))?;
    if row_num < 1 || row_num > size {
        return Err(format!("Row out of range: {}", row_num));
    }
    if col >= size {
        return Err(format!("Column out of range: {}", col_char));
    }
    Ok((row_num - 1, col))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_gtp() {
        assert_eq!(to_gtp(0, 0), "a1");
        assert_eq!(to_gtp(0, 3), "d1");
        assert_eq!(to_gtp(7, 7), "h8");
    }

    #[test]
    fn test_from_gtp() {
        assert_eq!(from_gtp("a1", 8), Ok((0, 0)));
        assert_eq!(from_gtp("D3", 8), Ok((2, 3)));
        assert_eq!(from_gtp("h8", 8), Ok((7, 7)));
    }

    #[test]
    fn test_from_gtp_errors() {
        assert!(from_gtp("z1", 8).is_err());
        assert!(from_gtp("a9", 8).is_err());
        assert!(from_gtp("a0", 8).is_err());
    }

    #[test]
    fn test_roundtrip() {
        for row in 0..8 {
            for col in 0..8 {
                let s = to_gtp(row, col);
                assert_eq!(from_gtp(&s, 8), Ok((row, col)));
            }
        }
    }
}
