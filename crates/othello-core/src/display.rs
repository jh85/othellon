use crate::coord::col_to_letter;

pub fn display_board(board_array: &[u8], size: usize) -> String {
    let mut s = String::new();
    s.push_str("  ");
    for col in 0..size {
        s.push(' ');
        s.push(col_to_letter(col));
    }
    s.push('\n');

    for row in 0..size {
        s.push_str(&format!("{:2}", row + 1));
        for col in 0..size {
            let cell = board_array[row * size + col];
            let c = match cell {
                1 => 'X',
                2 => 'O',
                _ => '.',
            };
            s.push(' ');
            s.push(c);
        }
        s.push('\n');
    }
    s
}
