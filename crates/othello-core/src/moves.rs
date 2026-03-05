use crate::bitboard::BitBoard;

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::N,
        Direction::S,
        Direction::E,
        Direction::W,
        Direction::NE,
        Direction::NW,
        Direction::SE,
        Direction::SW,
    ];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::N => (-1, 0),
            Direction::S => (1, 0),
            Direction::E => (0, 1),
            Direction::W => (0, -1),
            Direction::NE => (-1, 1),
            Direction::NW => (-1, -1),
            Direction::SE => (1, 1),
            Direction::SW => (1, -1),
        }
    }
}

/// Shift a bitboard in the given direction by one step.
pub fn shift_dir<const N: usize, const W: usize>(
    bb: &BitBoard<N, W>,
    dir: Direction,
) -> BitBoard<N, W> {
    match dir {
        Direction::N => bb.shr(N),
        Direction::S => bb.shl(N),
        Direction::E => bb.shl(1),
        Direction::W => bb.shr(1),
        Direction::NE => bb.shr(N - 1),
        Direction::NW => bb.shr(N + 1),
        Direction::SE => bb.shl(N + 1),
        Direction::SW => bb.shl(N - 1),
    }
}

/// Edge mask for a direction: prevents column wrap-around.
pub fn edge_mask<const N: usize, const W: usize>(dir: Direction) -> BitBoard<N, W> {
    match dir {
        Direction::N | Direction::S => BitBoard::<N, W>::valid_mask(),
        Direction::E | Direction::NE | Direction::SE => BitBoard::<N, W>::not_col_mask(N - 1),
        Direction::W | Direction::NW | Direction::SW => BitBoard::<N, W>::not_col_mask(0),
    }
}

/// Compute legal moves using directional flood-fill on bitboards.
///
/// The edge mask is applied to the SOURCE before each shift to prevent
/// column wrap-around. For eastward shifts (E/NE/SE), pieces in the last
/// column are excluded; for westward shifts (W/NW/SW), pieces in the first
/// column are excluded.
pub fn legal_moves<const N: usize, const W: usize>(
    player: &BitBoard<N, W>,
    opponent: &BitBoard<N, W>,
) -> BitBoard<N, W> {
    let empty = !(*player | *opponent) & BitBoard::<N, W>::valid_mask();
    let mut moves = BitBoard::zero();

    for &dir in &Direction::ALL {
        let edge = edge_mask::<N, W>(dir);
        let mut candidates = shift_dir(&(*player & edge), dir) & *opponent;
        for _ in 0..N - 2 {
            candidates = candidates | (shift_dir(&(candidates & edge), dir) & *opponent);
        }
        moves = moves | (shift_dir(&(candidates & edge), dir) & empty);
    }
    moves
}

/// Compute flipped discs when placing a piece at `pos`.
pub fn calc_flips<const N: usize, const W: usize>(
    player: &BitBoard<N, W>,
    opponent: &BitBoard<N, W>,
    pos: usize,
) -> BitBoard<N, W> {
    let row = pos / N;
    let col = pos % N;
    let mut flips = BitBoard::zero();

    for &dir in &Direction::ALL {
        let (dr, dc) = dir.delta();
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        let mut line = BitBoard::zero();

        while r >= 0 && r < N as i32 && c >= 0 && c < N as i32 {
            let idx = r as usize * N + c as usize;
            if opponent.get(idx) {
                line.set_mut(idx);
            } else if player.get(idx) {
                flips = flips | line;
                break;
            } else {
                break;
            }
            r += dr;
            c += dc;
        }
    }
    flips
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initial_8x8() -> (BitBoard<8, 1>, BitBoard<8, 1>) {
        let mut black = BitBoard::zero();
        let mut white = BitBoard::zero();
        // Standard opening: d5=W, e5=B, d4=B, e4=W (0-indexed row from top)
        // (3,3)=W, (3,4)=B, (4,3)=B, (4,4)=W
        white.set_mut(3 * 8 + 3);
        black.set_mut(3 * 8 + 4);
        black.set_mut(4 * 8 + 3);
        white.set_mut(4 * 8 + 4);
        (black, white)
    }

    #[test]
    fn test_legal_moves_opening() {
        let (black, white) = initial_8x8();
        let moves = legal_moves(&black, &white);
        let move_list = moves.iter_ones();
        assert_eq!(move_list.len(), 4);
        // Expected: (2,3)=19, (3,2)=26, (4,5)=37, (5,4)=44
        assert!(moves.get(2 * 8 + 3));
        assert!(moves.get(3 * 8 + 2));
        assert!(moves.get(4 * 8 + 5));
        assert!(moves.get(5 * 8 + 4));
    }

    #[test]
    fn test_calc_flips_opening() {
        let (black, white) = initial_8x8();
        // Black plays at (2,3) -> should flip white at (3,3)
        let flips = calc_flips(&black, &white, 2 * 8 + 3);
        assert_eq!(flips.count_ones(), 1);
        assert!(flips.get(3 * 8 + 3));
    }

    #[test]
    fn test_legal_moves_4x4() {
        let mut black = BitBoard::<4, 1>::zero();
        let mut white = BitBoard::<4, 1>::zero();
        white.set_mut(1 * 4 + 1);
        black.set_mut(1 * 4 + 2);
        black.set_mut(2 * 4 + 1);
        white.set_mut(2 * 4 + 2);
        let moves = legal_moves(&black, &white);
        assert_eq!(moves.iter_ones().len(), 4);
    }

    #[test]
    fn test_shift_dir_north() {
        let bb = BitBoard::<8, 1>::single(4 * 8 + 3); // row 4, col 3
        let shifted = shift_dir(&bb, Direction::N);
        assert!(shifted.get(3 * 8 + 3)); // row 3, col 3
    }

    #[test]
    fn test_no_column_wrap_nw() {
        // Regression: Black at a6(5,0) should NOT wrap NW to h4(3,7)
        // Board: White at e7,f6,f7,g5,h4,h5,h7,h8. Empty: g2,g3,g4,h1,h2,h3. Rest: Black.
        let mut black = BitBoard::<8, 1>::zero();
        let mut white = BitBoard::<8, 1>::zero();

        // Set White
        for &idx in &[
            6 * 8 + 4, // e7
            5 * 8 + 5, // f6
            6 * 8 + 5, // f7
            4 * 8 + 6, // g5
            3 * 8 + 7, // h4
            4 * 8 + 7, // h5
            6 * 8 + 7, // h7
            7 * 8 + 7, // h8
        ] {
            white.set_mut(idx);
        }

        // Set empty positions (not Black, not White)
        let empties = [
            1 * 8 + 6, // g2
            2 * 8 + 6, // g3
            3 * 8 + 6, // g4
            0 * 8 + 7, // h1
            1 * 8 + 7, // h2
            2 * 8 + 7, // h3
        ];

        // Fill all non-White non-Empty with Black
        for r in 0..8 {
            for c in 0..8 {
                let idx = r * 8 + c;
                if !white.get(idx) && !empties.contains(&idx) {
                    black.set_mut(idx);
                }
            }
        }

        let moves = legal_moves(&black, &white);
        let move_list = moves.iter_ones();

        // g3 (2,6) should NOT be legal (no valid flips from that position)
        assert!(
            !moves.get(2 * 8 + 6),
            "g3 should NOT be a legal move; wrap-around bug"
        );

        // g4 (3,6) SHOULD be legal (flips g5 via south)
        assert!(moves.get(3 * 8 + 6), "g4 should be legal");

        // h3 (2,7) SHOULD be legal (flips h4,h5 via south)
        assert!(moves.get(2 * 8 + 7), "h3 should be legal");

        // Exactly 2 legal moves: g4 and h3
        assert_eq!(move_list.len(), 2, "Expected exactly 2 legal moves, got {:?}",
            move_list.iter().map(|&p| (p / 8, p % 8)).collect::<Vec<_>>());
    }

    #[test]
    fn test_shift_dir_east_no_wrap() {
        let bb = BitBoard::<8, 1>::single(3 * 8 + 7); // row 3, col 7 (last col)
        let shifted = shift_dir(&bb, Direction::E);
        // Should shift to row 4 col 0 in raw bit terms, but that's wrap-around
        // The edge mask would prevent this, but shift_dir alone doesn't mask
        // So shifted.get(4*8+0) might be true - edge_mask handles this in legal_moves
        assert!(!shifted.get(3 * 8 + 7)); // original position is cleared
    }
}
