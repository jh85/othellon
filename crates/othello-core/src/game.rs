use crate::bitboard::BitBoard;
use crate::board::{Board, Color};
use crate::coord;
use crate::display;

#[derive(Clone)]
pub struct GameState<const N: usize, const W: usize> {
    board: Board<N, W>,
    current_color: Color,
    consecutive_passes: u8,
    game_over: bool,
    history: Vec<(Board<N, W>, Color, u8, bool)>,
}

impl<const N: usize, const W: usize> GameState<N, W> {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_color: Color::Black,
            consecutive_passes: 0,
            game_over: false,
            history: Vec::new(),
        }
    }

    pub fn board(&self) -> &Board<N, W> {
        &self.board
    }

    pub fn current_color(&self) -> Color {
        self.current_color
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn size(&self) -> usize {
        N
    }

    pub fn legal_moves(&self) -> BitBoard<N, W> {
        if self.game_over {
            BitBoard::zero()
        } else {
            self.board.legal_moves(self.current_color)
        }
    }

    pub fn must_pass(&self) -> bool {
        !self.game_over && self.legal_moves().is_zero()
    }

    pub fn play_move(&mut self, pos: usize) -> Result<(), String> {
        if self.game_over {
            return Err("Game is over".to_string());
        }
        let legal = self.legal_moves();
        if !legal.get(pos) {
            return Err(format!("Illegal move at position {}", pos));
        }
        self.history.push((
            self.board.clone(),
            self.current_color,
            self.consecutive_passes,
            self.game_over,
        ));
        self.board.play_move(self.current_color, pos);
        self.current_color = self.current_color.opposite();
        self.consecutive_passes = 0;
        self.check_game_over();
        Ok(())
    }

    pub fn pass_turn(&mut self) -> Result<(), String> {
        if self.game_over {
            return Err("Game is over".to_string());
        }
        if !self.legal_moves().is_zero() {
            return Err("Cannot pass when legal moves exist".to_string());
        }
        self.history.push((
            self.board.clone(),
            self.current_color,
            self.consecutive_passes,
            self.game_over,
        ));
        self.current_color = self.current_color.opposite();
        self.consecutive_passes += 1;
        self.check_game_over();
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), String> {
        if let Some((board, color, passes, over)) = self.history.pop() {
            self.board = board;
            self.current_color = color;
            self.consecutive_passes = passes;
            self.game_over = over;
            Ok(())
        } else {
            Err("No moves to undo".to_string())
        }
    }

    pub fn score(&self) -> (u32, u32) {
        self.board.score()
    }

    pub fn get_board_array(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(N * N);
        for row in 0..N {
            for col in 0..N {
                let idx = row * N + col;
                if self.board.black.get(idx) {
                    result.push(1);
                } else if self.board.white.get(idx) {
                    result.push(2);
                } else {
                    result.push(0);
                }
            }
        }
        result
    }

    pub fn get_legal_moves_list(&self) -> Vec<(usize, usize)> {
        self.legal_moves()
            .iter_ones()
            .iter()
            .map(|&pos| (pos / N, pos % N))
            .collect()
    }

    pub fn genmove_ai(&mut self, depth_offset: i32, rand_seed: u64) -> Result<Option<usize>, String> {
        if self.game_over {
            return Err("Game is over".to_string());
        }
        if self.must_pass() {
            self.pass_turn()?;
            return Ok(None);
        }
        match crate::ai::best_move(&self.board, self.current_color, depth_offset, rand_seed) {
            Some(pos) => {
                self.play_move(pos)?;
                Ok(Some(pos))
            }
            None => {
                self.pass_turn()?;
                Ok(None)
            }
        }
    }

    fn check_game_over(&mut self) {
        if self.consecutive_passes >= 2 {
            self.game_over = true;
            return;
        }
        let current_moves = self.board.legal_moves(self.current_color);
        if current_moves.is_zero() {
            let opponent_moves = self.board.legal_moves(self.current_color.opposite());
            if opponent_moves.is_zero() {
                self.game_over = true;
            }
        }
    }
}

// --- Dynamic dispatch wrapper ---

macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match &$self.inner {
            GameInner::B4(g) => g.$method($($arg),*),
            GameInner::B6(g) => g.$method($($arg),*),
            GameInner::B8(g) => g.$method($($arg),*),
            GameInner::B10(g) => g.$method($($arg),*),
            GameInner::B12(g) => g.$method($($arg),*),
            GameInner::B14(g) => g.$method($($arg),*),
            GameInner::B16(g) => g.$method($($arg),*),
            GameInner::B18(g) => g.$method($($arg),*),
            GameInner::B20(g) => g.$method($($arg),*),
        }
    };
}

macro_rules! dispatch_mut {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match &mut $self.inner {
            GameInner::B4(g) => g.$method($($arg),*),
            GameInner::B6(g) => g.$method($($arg),*),
            GameInner::B8(g) => g.$method($($arg),*),
            GameInner::B10(g) => g.$method($($arg),*),
            GameInner::B12(g) => g.$method($($arg),*),
            GameInner::B14(g) => g.$method($($arg),*),
            GameInner::B16(g) => g.$method($($arg),*),
            GameInner::B18(g) => g.$method($($arg),*),
            GameInner::B20(g) => g.$method($($arg),*),
        }
    };
}

#[allow(clippy::large_enum_variant)]
enum GameInner {
    B4(GameState<4, 1>),
    B6(GameState<6, 1>),
    B8(GameState<8, 1>),
    B10(GameState<10, 2>),
    B12(GameState<12, 3>),
    B14(GameState<14, 4>),
    B16(GameState<16, 4>),
    B18(GameState<18, 6>),
    B20(GameState<20, 7>),
}

pub struct Game {
    inner: GameInner,
}

impl Game {
    pub fn new(size: usize) -> Result<Self, String> {
        let inner = match size {
            4 => GameInner::B4(GameState::new()),
            6 => GameInner::B6(GameState::new()),
            8 => GameInner::B8(GameState::new()),
            10 => GameInner::B10(GameState::new()),
            12 => GameInner::B12(GameState::new()),
            14 => GameInner::B14(GameState::new()),
            16 => GameInner::B16(GameState::new()),
            18 => GameInner::B18(GameState::new()),
            20 => GameInner::B20(GameState::new()),
            _ => {
                return Err(format!(
                    "Unsupported board size: {}. Must be even, 4-20.",
                    size
                ))
            }
        };
        Ok(Self { inner })
    }

    pub fn size(&self) -> usize {
        dispatch!(self, size)
    }

    pub fn current_color(&self) -> Color {
        dispatch!(self, current_color)
    }

    pub fn is_game_over(&self) -> bool {
        dispatch!(self, is_game_over)
    }

    pub fn must_pass(&self) -> bool {
        dispatch!(self, must_pass)
    }

    pub fn play_move(&mut self, row: usize, col: usize) -> Result<(), String> {
        let size = self.size();
        let pos = row * size + col;
        dispatch_mut!(self, play_move, pos)
    }

    pub fn play_move_gtp(&mut self, color: Color, vertex: &str) -> Result<(), String> {
        let size = self.size();
        if vertex.to_lowercase() == "pass" {
            return self.pass_turn();
        }
        let (row, col) = coord::from_gtp(vertex, size)?;
        if color != self.current_color() {
            return Err(format!("Not {}'s turn", color.name()));
        }
        self.play_move(row, col)
    }

    pub fn pass_turn(&mut self) -> Result<(), String> {
        dispatch_mut!(self, pass_turn)
    }

    pub fn undo(&mut self) -> Result<(), String> {
        dispatch_mut!(self, undo)
    }

    pub fn score(&self) -> (u32, u32) {
        dispatch!(self, score)
    }

    pub fn get_board_array(&self) -> Vec<u8> {
        dispatch!(self, get_board_array)
    }

    pub fn get_legal_moves_list(&self) -> Vec<(usize, usize)> {
        dispatch!(self, get_legal_moves_list)
    }

    pub fn genmove_ai(&mut self, depth_offset: i32, rand_seed: u64) -> Result<Option<(usize, usize)>, String> {
        let size = self.size();
        let pos_opt = dispatch_mut!(self, genmove_ai, depth_offset, rand_seed)?;
        Ok(pos_opt.map(|p| (p / size, p % size)))
    }

    pub fn showboard(&self) -> String {
        let arr = self.get_board_array();
        display::display_board(&arr, self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_8x8_opening() {
        let gs = GameState::<8, 1>::new();
        assert_eq!(gs.current_color(), Color::Black);
        assert!(!gs.is_game_over());
        let moves = gs.get_legal_moves_list();
        assert_eq!(moves.len(), 4);
    }

    #[test]
    fn test_game_state_play_and_undo() {
        let mut gs = GameState::<8, 1>::new();
        gs.play_move(2 * 8 + 3).unwrap(); // d3
        assert_eq!(gs.current_color(), Color::White);
        assert_eq!(gs.score(), (4, 1));
        gs.undo().unwrap();
        assert_eq!(gs.current_color(), Color::Black);
        assert_eq!(gs.score(), (2, 2));
    }

    #[test]
    fn test_game_dynamic_dispatch() {
        let mut game = Game::new(8).unwrap();
        assert_eq!(game.size(), 8);
        assert_eq!(game.current_color(), Color::Black);
        let moves = game.get_legal_moves_list();
        assert_eq!(moves.len(), 4);
        game.play_move(2, 3).unwrap();
        assert_eq!(game.current_color(), Color::White);
    }

    #[test]
    fn test_game_4x4_to_completion() {
        let mut gs = GameState::<4, 1>::new();
        let mut move_count = 0;
        while !gs.is_game_over() {
            let moves = gs.legal_moves();
            let move_list = moves.iter_ones();
            if move_list.is_empty() {
                gs.pass_turn().unwrap();
            } else {
                gs.play_move(move_list[0]).unwrap();
            }
            move_count += 1;
            assert!(move_count < 100, "Game should end in fewer than 100 moves");
        }
        let (b, w) = gs.score();
        assert!(b + w > 4); // More pieces than start
    }

    #[test]
    fn test_game_various_sizes() {
        for &size in &[4, 6, 8, 10, 12, 14, 16, 18, 20] {
            let game = Game::new(size).unwrap();
            assert_eq!(game.size(), size);
            let moves = game.get_legal_moves_list();
            assert_eq!(moves.len(), 4, "Size {} should have 4 opening moves", size);
        }
    }

    #[test]
    fn test_invalid_size() {
        assert!(Game::new(3).is_err());
        assert!(Game::new(7).is_err());
        assert!(Game::new(22).is_err());
    }
}
