use std::io::{self, BufRead, Write};

use othello_core::{Color, Game};
use othello_core::coord;

use crate::parser::{self, GtpCommand};

const KNOWN_COMMANDS: &[&str] = &[
    "protocol_version",
    "name",
    "version",
    "known_command",
    "list_commands",
    "quit",
    "boardsize",
    "clear_board",
    "komi",
    "play",
    "genmove",
    "showboard",
    "undo",
    "set_game",
    "list_games",
    "gogui-rules_game_id",
    "gogui-rules_board_size",
    "gogui-rules_board",
    "gogui-rules_legal_moves",
    "gogui-rules_side_to_move",
    "gogui-rules_final_result",
];

struct GtpEngine {
    game: Game,
}

impl GtpEngine {
    fn new() -> Self {
        Self {
            game: Game::new(8).unwrap(),
        }
    }

    fn handle(&mut self, cmd: &GtpCommand) -> Result<String, String> {
        match cmd.name.as_str() {
            "protocol_version" => Ok("2".to_string()),
            "name" => Ok("othello-gtp".to_string()),
            "version" => Ok("0.1.0".to_string()),
            "known_command" => {
                let name = cmd.args.first().map(|s| s.as_str()).unwrap_or("");
                Ok(if KNOWN_COMMANDS.contains(&name) {
                    "true"
                } else {
                    "false"
                }
                .to_string())
            }
            "list_commands" => Ok(KNOWN_COMMANDS.join("\n")),
            "quit" => Ok(String::new()),
            "boardsize" => {
                let size: usize = cmd
                    .args
                    .first()
                    .ok_or("missing size")?
                    .parse()
                    .map_err(|_| "invalid size".to_string())?;
                if size % 2 != 0 || size < 4 || size > 20 {
                    return Err(format!("unsupported size {}", size));
                }
                self.game = Game::new(size).map_err(|e| e.to_string())?;
                Ok(String::new())
            }
            "clear_board" => {
                let size = self.game.size();
                self.game = Game::new(size).unwrap();
                Ok(String::new())
            }
            "komi" => Ok(String::new()), // ignored for Othello
            "play" => {
                let color_str = cmd.args.first().ok_or("missing color")?;
                let vertex = cmd.args.get(1).ok_or("missing vertex")?;
                let color = parse_color(color_str)?;
                self.game
                    .play_move_gtp(color, vertex)
                    .map_err(|e| e.to_string())?;
                Ok(String::new())
            }
            "genmove" => {
                let color_str = cmd.args.first().ok_or("missing color")?;
                let color = parse_color(color_str)?;
                if color != self.game.current_color() {
                    return Err(format!("not {}'s turn", color.name()));
                }
                if self.game.is_game_over() {
                    return Err("game is over".to_string());
                }
                let rand_seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);
                match self.game.genmove_ai(0, rand_seed).map_err(|e| e.to_string())? {
                    Some((row, col)) => Ok(coord::to_gtp(row, col)),
                    None => Ok("pass".to_string()),
                }
            }
            "showboard" => Ok(format!("\n{}", self.game.showboard())),
            "undo" => {
                self.game.undo().map_err(|e| e.to_string())?;
                Ok(String::new())
            }
            "set_game" => {
                let game_name = cmd.args.first().map(|s| s.as_str()).unwrap_or("");
                if game_name.eq_ignore_ascii_case("othello")
                    || game_name.eq_ignore_ascii_case("reversi")
                {
                    Ok(String::new())
                } else {
                    Err(format!("unsupported game: {}", game_name))
                }
            }
            "list_games" => Ok("Othello".to_string()),
            "gogui-rules_game_id" => Ok("Othello".to_string()),
            "gogui-rules_board_size" => Ok(self.game.size().to_string()),
            "gogui-rules_board" => Ok(format!("\n{}", self.game.showboard())),
            "gogui-rules_legal_moves" => {
                let moves = self.game.get_legal_moves_list();
                let strs: Vec<String> =
                    moves.iter().map(|&(r, c)| coord::to_gtp(r, c)).collect();
                Ok(strs.join(" "))
            }
            "gogui-rules_side_to_move" => Ok(self.game.current_color().name().to_string()),
            "gogui-rules_final_result" => {
                if !self.game.is_game_over() {
                    return Ok("0".to_string());
                }
                let (b, w) = self.game.score();
                Ok(if b > w {
                    format!("B+{}", b - w)
                } else if w > b {
                    format!("W+{}", w - b)
                } else {
                    "0".to_string()
                })
            }
            _ => Err(format!("unknown command: {}", cmd.name)),
        }
    }
}

fn parse_color(s: &str) -> Result<Color, String> {
    match s.to_lowercase().as_str() {
        "b" | "black" => Ok(Color::Black),
        "w" | "white" => Ok(Color::White),
        _ => Err(format!("invalid color: {}", s)),
    }
}

fn format_response(id: Option<u32>, result: Result<String, String>) -> String {
    match result {
        Ok(msg) => match id {
            Some(n) => format!("={} {}\n\n", n, msg),
            None => format!("= {}\n\n", msg),
        },
        Err(msg) => match id {
            Some(n) => format!("?{} {}\n\n", n, msg),
            None => format!("? {}\n\n", msg),
        },
    }
}

pub fn run() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut engine = GtpEngine::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let cmd = match parser::parse_line(&line) {
            Some(c) => c,
            None => continue,
        };

        let is_quit = cmd.name == "quit";
        let result = engine.handle(&cmd);
        let response = format_response(cmd.id, result);

        write!(stdout, "{}", response).ok();
        stdout.flush().ok();

        if is_quit {
            break;
        }
    }
}
