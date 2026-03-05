use crate::bitboard::BitBoard;
use crate::moves::{calc_flips, legal_moves};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Color::Black => "black",
            Color::White => "white",
        }
    }
}

#[derive(Clone)]
pub struct Board<const N: usize, const W: usize> {
    pub black: BitBoard<N, W>,
    pub white: BitBoard<N, W>,
}

impl<const N: usize, const W: usize> Board<N, W> {
    pub fn new() -> Self {
        let mut black = BitBoard::zero();
        let mut white = BitBoard::zero();
        let mid = N / 2;
        white.set_mut((mid - 1) * N + (mid - 1));
        black.set_mut((mid - 1) * N + mid);
        black.set_mut(mid * N + (mid - 1));
        white.set_mut(mid * N + mid);
        Self { black, white }
    }

    pub fn legal_moves(&self, color: Color) -> BitBoard<N, W> {
        let (player, opponent) = match color {
            Color::Black => (&self.black, &self.white),
            Color::White => (&self.white, &self.black),
        };
        legal_moves(player, opponent)
    }

    pub fn play_move(&mut self, color: Color, pos: usize) {
        let (player, opponent) = match color {
            Color::Black => (&self.black, &self.white),
            Color::White => (&self.white, &self.black),
        };
        let flips = calc_flips(player, opponent, pos);
        self.black ^= flips;
        self.white ^= flips;
        match color {
            Color::Black => self.black.set_mut(pos),
            Color::White => self.white.set_mut(pos),
        }
    }

    pub fn score(&self) -> (u32, u32) {
        (self.black.count_ones(), self.white.count_ones())
    }

    pub fn cell_at(&self, row: usize, col: usize) -> Option<Color> {
        let idx = row * N + col;
        if self.black.get(idx) {
            Some(Color::Black)
        } else if self.white.get(idx) {
            Some(Color::White)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_board_8x8() {
        let board = Board::<8, 1>::new();
        assert_eq!(board.cell_at(3, 3), Some(Color::White));
        assert_eq!(board.cell_at(3, 4), Some(Color::Black));
        assert_eq!(board.cell_at(4, 3), Some(Color::Black));
        assert_eq!(board.cell_at(4, 4), Some(Color::White));
        assert_eq!(board.score(), (2, 2));
    }

    #[test]
    fn test_play_move_8x8() {
        let mut board = Board::<8, 1>::new();
        // Black plays at (2,3) - should flip white at (3,3)
        let legal = board.legal_moves(Color::Black);
        assert!(legal.get(2 * 8 + 3));
        board.play_move(Color::Black, 2 * 8 + 3);
        assert_eq!(board.cell_at(2, 3), Some(Color::Black));
        assert_eq!(board.cell_at(3, 3), Some(Color::Black)); // was white, now flipped
        assert_eq!(board.score(), (4, 1));
    }

    #[test]
    fn test_initial_board_4x4() {
        let board = Board::<4, 1>::new();
        assert_eq!(board.cell_at(1, 1), Some(Color::White));
        assert_eq!(board.cell_at(1, 2), Some(Color::Black));
        assert_eq!(board.cell_at(2, 1), Some(Color::Black));
        assert_eq!(board.cell_at(2, 2), Some(Color::White));
        assert_eq!(board.score(), (2, 2));
    }
}
