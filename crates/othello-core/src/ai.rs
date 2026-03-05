use crate::board::{Board, Color};
use crate::moves::legal_moves;

/// Categorize a position for move ordering.
/// 0 = corner (best), 1 = edge, 2 = interior, 3 = corner-adjacent (worst)
fn move_priority<const N: usize>(pos: usize) -> u8 {
    let row = pos / N;
    let col = pos % N;

    // Corner
    if (row == 0 || row == N - 1) && (col == 0 || col == N - 1) {
        return 0;
    }

    // Corner-adjacent (X-squares and C-squares)
    if N >= 4 {
        let corner_adj: [(usize, usize); 12] = [
            (1, 1),
            (0, 1),
            (1, 0),
            (1, N - 2),
            (0, N - 2),
            (1, N - 1),
            (N - 2, 1),
            (N - 1, 1),
            (N - 2, 0),
            (N - 2, N - 2),
            (N - 1, N - 2),
            (N - 2, N - 1),
        ];
        for &(ar, ac) in &corner_adj {
            if row == ar && col == ac {
                return 3;
            }
        }
    }

    // Edge
    if row == 0 || row == N - 1 || col == 0 || col == N - 1 {
        return 1;
    }

    // Interior
    2
}

/// Evaluate board position from the perspective of `color`.
fn evaluate<const N: usize, const W: usize>(board: &Board<N, W>, color: Color) -> i32 {
    let (player, opponent) = match color {
        Color::Black => (&board.black, &board.white),
        Color::White => (&board.white, &board.black),
    };

    let player_moves = legal_moves(player, opponent);
    let opponent_moves = legal_moves(opponent, player);

    // Game over
    if player_moves.is_zero() && opponent_moves.is_zero() {
        let diff = player.count_ones() as i32 - opponent.count_ones() as i32;
        return if diff > 0 {
            10000 + diff
        } else if diff < 0 {
            -10000 + diff
        } else {
            0
        };
    }

    let mut score = 0i32;

    // Disc difference (weight 1)
    score += player.count_ones() as i32 - opponent.count_ones() as i32;

    // Mobility (weight 5)
    score += 5 * (player_moves.count_ones() as i32 - opponent_moves.count_ones() as i32);

    // Corner control (weight 25)
    let corners = [0, N - 1, (N - 1) * N, (N - 1) * N + N - 1];
    for &c in &corners {
        if player.get(c) {
            score += 25;
        } else if opponent.get(c) {
            score -= 25;
        }
    }

    // Corner-adjacent penalties (only when corner is empty)
    if N >= 4 {
        let corner_adj: [(usize, usize, [usize; 2]); 4] = [
            (0, 1 * N + 1, [1, N]),
            (N - 1, 1 * N + (N - 2), [N - 2, N + N - 1]),
            ((N - 1) * N, (N - 2) * N + 1, [(N - 1) * N + 1, (N - 2) * N]),
            (
                (N - 1) * N + N - 1,
                (N - 2) * N + (N - 2),
                [(N - 1) * N + N - 2, (N - 2) * N + N - 1],
            ),
        ];

        for &(corner, x_sq, c_sqs) in &corner_adj {
            if !player.get(corner) && !opponent.get(corner) {
                // X-square penalty (-20)
                if player.get(x_sq) {
                    score -= 20;
                } else if opponent.get(x_sq) {
                    score += 20;
                }
                // C-square penalty (-10)
                for &c_sq in &c_sqs {
                    if player.get(c_sq) {
                        score -= 10;
                    } else if opponent.get(c_sq) {
                        score += 10;
                    }
                }
            }
        }
    }

    // Edge bonus (weight 2), excluding corners
    for col in 1..N - 1 {
        // Top edge
        if player.get(col) {
            score += 2;
        } else if opponent.get(col) {
            score -= 2;
        }
        // Bottom edge
        let idx = (N - 1) * N + col;
        if player.get(idx) {
            score += 2;
        } else if opponent.get(idx) {
            score -= 2;
        }
    }
    for row in 1..N - 1 {
        // Left edge
        let idx = row * N;
        if player.get(idx) {
            score += 2;
        } else if opponent.get(idx) {
            score -= 2;
        }
        // Right edge
        let idx = row * N + N - 1;
        if player.get(idx) {
            score += 2;
        } else if opponent.get(idx) {
            score -= 2;
        }
    }

    score
}

/// Select search depth based on board size and number of empty squares.
fn search_depth(n: usize, empties: u32) -> i32 {
    if empties <= 12 {
        return empties as i32;
    }
    match n {
        4 => 12,
        6 => 8,
        8 => 6,
        10 => 5,
        12 => 4,
        14 => 3,
        _ => 3,
    }
}

/// Negamax search with alpha-beta pruning.
fn negamax<const N: usize, const W: usize>(
    board: &Board<N, W>,
    color: Color,
    depth: i32,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    let (player, opponent) = match color {
        Color::Black => (&board.black, &board.white),
        Color::White => (&board.white, &board.black),
    };

    let player_legal = legal_moves(player, opponent);
    let opponent_legal = legal_moves(opponent, player);

    // Terminal: both sides have no moves
    if player_legal.is_zero() && opponent_legal.is_zero() {
        let diff = player.count_ones() as i32 - opponent.count_ones() as i32;
        return if diff > 0 {
            10000 + diff
        } else if diff < 0 {
            -10000 + diff
        } else {
            0
        };
    }

    // Leaf: return static evaluation
    if depth <= 0 {
        return evaluate(board, color);
    }

    // Must pass: opponent's turn, don't decrement depth
    if player_legal.is_zero() {
        return -negamax(board, color.opposite(), depth, -beta, -alpha);
    }

    // Generate and order moves
    let mut moves: Vec<usize> = player_legal.iter_ones();
    moves.sort_by_key(|&pos| move_priority::<N>(pos));

    let mut best = i32::MIN + 1;

    for &pos in &moves {
        let mut new_board = board.clone();
        new_board.play_move(color, pos);
        let score = -negamax(&new_board, color.opposite(), depth - 1, -beta, -alpha);
        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    best
}

/// Score window: moves within this margin of the best are considered equally good.
const SCORE_WINDOW: i32 = 3;

/// Simple xorshift64 PRNG (no external dependency needed).
fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

/// Find the best move for `color` on the given board.
/// `depth_offset` adjusts search depth: 0 = default (L2), -1 = L1, +1 = L3.
/// `rand_seed` is used to randomly select among equally-good moves.
/// Returns `None` if there are no legal moves (caller should pass).
pub fn best_move<const N: usize, const W: usize>(
    board: &Board<N, W>,
    color: Color,
    depth_offset: i32,
    rand_seed: u64,
) -> Option<usize> {
    let (player, opponent) = match color {
        Color::Black => (&board.black, &board.white),
        Color::White => (&board.white, &board.black),
    };

    let player_legal = legal_moves(player, opponent);
    if player_legal.is_zero() {
        return None;
    }

    let total = (N * N) as u32;
    let occupied = board.black.count_ones() + board.white.count_ones();
    let empties = total - occupied;
    let depth = (search_depth(N, empties) + depth_offset).max(1);

    let mut moves: Vec<usize> = player_legal.iter_ones();
    moves.sort_by_key(|&pos| move_priority::<N>(pos));

    let mut scored: Vec<(usize, i32)> = Vec::with_capacity(moves.len());
    let mut best_score = i32::MIN + 1;
    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX;

    for &pos in &moves {
        let mut new_board = board.clone();
        new_board.play_move(color, pos);
        let score = -negamax(&new_board, color.opposite(), depth - 1, -beta, -alpha);
        scored.push((pos, score));
        if score > best_score {
            best_score = score;
        }
        if score > alpha {
            alpha = score;
        }
    }

    // Collect all moves within SCORE_WINDOW of the best
    let threshold = best_score - SCORE_WINDOW;
    let candidates: Vec<usize> = scored
        .iter()
        .filter(|(_, s)| *s >= threshold)
        .map(|(p, _)| *p)
        .collect();

    // Pick randomly among candidates
    let rng = xorshift64(rand_seed.wrapping_add(1));
    let idx = (rng % candidates.len() as u64) as usize;
    Some(candidates[idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::BitBoard;

    #[test]
    fn test_ai_returns_legal_move_4x4() {
        let board = Board::<4, 1>::new();
        let legal = board.legal_moves(Color::Black);
        let mv = best_move::<4, 1>(&board, Color::Black, 0, 42).unwrap();
        assert!(legal.get(mv), "AI should return a legal move");
    }

    #[test]
    fn test_ai_returns_legal_move_8x8() {
        let board = Board::<8, 1>::new();
        let legal = board.legal_moves(Color::Black);
        let mv = best_move::<8, 1>(&board, Color::Black, 0, 42).unwrap();
        assert!(legal.get(mv), "AI should return a legal move");
    }

    #[test]
    fn test_evaluate_balanced_opening() {
        let board = Board::<8, 1>::new();
        let score = evaluate::<8, 1>(&board, Color::Black);
        assert!(
            score.abs() < 50,
            "Opening eval should be near zero, got {}",
            score
        );
    }

    #[test]
    fn test_evaluate_game_over_win() {
        let mut black = BitBoard::<4, 1>::zero();
        for i in 0..16 {
            black.set_mut(i);
        }
        let board = Board {
            black,
            white: BitBoard::zero(),
        };
        let score = evaluate::<4, 1>(&board, Color::Black);
        assert!(score > 10000, "Black should have winning score, got {}", score);
    }

    #[test]
    fn test_evaluate_game_over_loss() {
        let mut white = BitBoard::<4, 1>::zero();
        for i in 0..16 {
            white.set_mut(i);
        }
        let board = Board {
            black: BitBoard::zero(),
            white,
        };
        let score = evaluate::<4, 1>(&board, Color::Black);
        assert!(score < -10000, "Black should have losing score, got {}", score);
    }

    #[test]
    fn test_ai_picks_corner_when_only_option_matters() {
        // Build a near-endgame 4x4 board where corner is the decisive move
        // On small boards with deep search, corner advantage is overwhelming
        let mut black = BitBoard::<4, 1>::zero();
        let mut white = BitBoard::<4, 1>::zero();
        // Row 0: empty, W, W, B
        white.set_mut(1);
        white.set_mut(2);
        black.set_mut(3);
        // Row 1: empty, B, W, B
        black.set_mut(5);
        white.set_mut(6);
        black.set_mut(7);
        // Row 2: B, B, B, B
        for c in 0..4 {
            black.set_mut(8 + c);
        }
        // Row 3: B, B, B, empty
        for c in 0..3 {
            black.set_mut(12 + c);
        }

        let board = Board { black, white };
        let legal = board.legal_moves(Color::Black);

        // Corner (0,0) should be legal and the best move
        if legal.get(0) {
            let mv = best_move::<4, 1>(&board, Color::Black, 0, 42).unwrap();
            assert_eq!(mv, 0, "AI should pick the corner at (0,0)");
        }
    }

    #[test]
    fn test_ai_no_legal_moves() {
        // Board completely filled with black - no legal moves for either side
        let mut black = BitBoard::<4, 1>::zero();
        for i in 0..16 {
            black.set_mut(i);
        }
        let board = Board {
            black,
            white: BitBoard::zero(),
        };
        assert!(best_move::<4, 1>(&board, Color::White, 0, 42).is_none());
    }

    #[test]
    fn test_move_priority_corners() {
        assert_eq!(move_priority::<8>(0), 0);
        assert_eq!(move_priority::<8>(7), 0);
        assert_eq!(move_priority::<8>(56), 0);
        assert_eq!(move_priority::<8>(63), 0);
    }

    #[test]
    fn test_move_priority_edges() {
        assert_eq!(move_priority::<8>(4), 1); // top edge (0,4)
        assert_eq!(move_priority::<8>(2 * 8), 1); // left edge (2,0)
        assert_eq!(move_priority::<8>(60), 1); // bottom edge (7,4)
        assert_eq!(move_priority::<8>(3 * 8 + 7), 1); // right edge (3,7)
    }

    #[test]
    fn test_move_priority_corner_adjacent() {
        assert_eq!(move_priority::<8>(1 * 8 + 1), 3); // X-square near (0,0)
        assert_eq!(move_priority::<8>(0 * 8 + 1), 3); // C-square near (0,0)
        assert_eq!(move_priority::<8>(1 * 8 + 0), 3); // C-square near (0,0)
    }

    #[test]
    fn test_move_priority_interior() {
        assert_eq!(move_priority::<8>(3 * 8 + 3), 2);
        assert_eq!(move_priority::<8>(4 * 8 + 4), 2);
    }
}
