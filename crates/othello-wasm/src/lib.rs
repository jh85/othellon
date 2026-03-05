use wasm_bindgen::prelude::*;
use othello_core::{Color, Game};

#[wasm_bindgen]
pub struct WasmGame {
    game: Game,
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize) -> Result<WasmGame, JsValue> {
        Game::new(size)
            .map(|game| WasmGame { game })
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn size(&self) -> usize {
        self.game.size()
    }

    /// Returns board as flat array: 0=empty, 1=black, 2=white
    pub fn get_board(&self) -> Vec<u8> {
        self.game.get_board_array()
    }

    /// Returns legal moves as flat array: [row0, col0, row1, col1, ...]
    pub fn get_legal_moves(&self) -> Vec<u32> {
        self.game
            .get_legal_moves_list()
            .iter()
            .flat_map(|&(r, c)| vec![r as u32, c as u32])
            .collect()
    }

    /// 0 = black, 1 = white
    pub fn current_player(&self) -> u8 {
        match self.game.current_color() {
            Color::Black => 0,
            Color::White => 1,
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game.is_game_over()
    }

    pub fn must_pass(&self) -> bool {
        self.game.must_pass()
    }

    pub fn play_move(&mut self, row: usize, col: usize) -> Result<(), JsValue> {
        self.game
            .play_move(row, col)
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn pass_turn(&mut self) -> Result<(), JsValue> {
        self.game.pass_turn().map_err(|e| JsValue::from_str(&e))
    }

    pub fn undo(&mut self) -> Result<(), JsValue> {
        self.game.undo().map_err(|e| JsValue::from_str(&e))
    }

    /// Returns [black_score, white_score]
    pub fn get_score(&self) -> Vec<u32> {
        let (b, w) = self.game.score();
        vec![b, w]
    }

    /// Pick and play the best move using AI search.
    /// `level`: 0=random, 1=shallow, 2=default, 3=deep, 4=deeper.
    /// Returns [row, col] or empty if pass.
    pub fn play_ai_move(&mut self, level: u32) -> Result<Vec<u32>, JsValue> {
        if level == 0 {
            return self.play_random_move();
        }
        let depth_offset = level as i32 - 2; // L1=-1, L2=0, L3=+1
        let rand_seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        match self.game.genmove_ai(depth_offset, rand_seed).map_err(|e| JsValue::from_str(&e))? {
            Some((row, col)) => Ok(vec![row as u32, col as u32]),
            None => Ok(vec![]),
        }
    }

    /// Pick and play a random legal move. Returns [row, col] or empty if pass.
    pub fn play_random_move(&mut self) -> Result<Vec<u32>, JsValue> {
        let moves = self.game.get_legal_moves_list();
        if moves.is_empty() {
            self.game.pass_turn().map_err(|e| JsValue::from_str(&e))?;
            Ok(vec![])
        } else {
            let idx = (js_sys::Math::random() * moves.len() as f64) as usize;
            let (row, col) = moves[idx];
            self.game
                .play_move(row, col)
                .map_err(|e| JsValue::from_str(&e))?;
            Ok(vec![row as u32, col as u32])
        }
    }
}
