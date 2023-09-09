pub(crate) mod alpha;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::Window;
use ts_rs::TS;

use crate::bot::ChessBot;

#[derive(TS, Serialize, Deserialize, Clone, Copy)]
#[ts(export, export_to = "../src/bindings/Position.ts")]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub fn new(x: i8, y: i8) -> Position {
        Position { x, y }
    }

    pub fn to_index(&self) -> i8 {
        self.x + self.y * 8
    }

    pub fn from_index(index: i8) -> Position {
        Position {
            x: index % 8,
            y: index / 8,
        }
    }

    pub fn from_x_y(x: i8, y: i8) -> Position {
        Position { x, y }
    }

    pub fn to_algebraic(&self) -> String {
        let mut algebraic = String::new();
        algebraic.push((self.x + 97) as u8 as char);
        algebraic.push((self.y + 49) as u8 as char);
        algebraic
    }

    pub fn from_algebraic(algebraic: String) -> Position {
        let mut chars = algebraic.as_str().chars();
        let x = chars.next().unwrap() as u8 - 97;
        let y = chars.next().unwrap() as u8 - 49;
        Position::new(x as i8, y as i8)
    }

    pub fn compare(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn mov(&self, delta: Position) -> Option<Position> {
        let x = self.x + delta.x;
        let y = self.y + delta.y;
        if x < 0 || x > 7 || y < 0 || y > 7 {
            return None;
        }
        Some(Position::new(x, y))
    }

    pub fn none() -> Position {
        Position { x: -1, y: -1 }
    }
}

#[derive(TS, Serialize, Deserialize, Clone, Copy)]
#[ts(export, export_to = "../src/bindings/Move.ts")]
pub struct Move {
    pub to: Position,
    pub from: Position,
}

impl Move {
    pub fn validate(from: Position, to: Position) -> Option<Move> {
        let x = from.x + to.x;
        let y = from.y + to.y;

        if x < 0 || x > 7 || y < 0 || y > 7 {
            return None;
        }

        Some(Move { to, from })
    }

	pub fn from (from: Position, to: Position) -> Move {
		Move { to, from }
	}
}

#[derive(TS, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[ts(export, export_to = "../src/bindings/GameState.ts")]
pub enum GameState {
    Playing,
    WhiteInCheck,
    BlackInCheck,
    WhiteInCheckmate,
    BlackInCheckmate,
    Stalemate,
    None,
}

#[derive(TS, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[ts(export, export_to = "../src/bindings/PieceKind.ts")]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None,
}

#[derive(TS, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[ts(export, export_to = "../src/bindings/PieceColor.ts")]
pub enum PieceColor {
    White,
    Black,
    None,
    Both,
}

impl PieceColor {
    pub fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
            _ => PieceColor::None,
        }
    }
}

#[derive(TS, Serialize, Deserialize, Clone, Copy)]
#[ts(export, export_to = "../src/bindings/Piece.ts")]
pub struct Piece {
    pub kind: PieceKind,
    pub color: PieceColor,
}

#[derive(TS, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "../src/bindings/Board.ts")]
pub struct Board {
    pub pieces: Vec<Piece>,
    pub turn: PieceColor,
    pub selected_piece: Option<Position>,
    pub game_state: GameState,
    pub white_castle_kingside: bool,
    pub white_castle_queenside: bool,
    pub black_castle_kingside: bool,
    pub black_castle_queenside: bool,
    pub en_passant: Option<Position>,
    pub bot: PieceColor,
}

impl Board {
    pub fn empty_board() -> Board {
        Board {
            pieces: vec![
                Piece {
                    kind: PieceKind::None,
                    color: PieceColor::None
                };
                64
            ],
            turn: PieceColor::White,
            selected_piece: None,
            game_state: GameState::Playing,
            white_castle_kingside: false,
            white_castle_queenside: false,
            black_castle_kingside: false,
            black_castle_queenside: false,
            en_passant: None,
            bot: PieceColor::None,
        }
    }

    pub fn default_board() -> Board {
        let mut pieces: Vec<Piece> = Vec::new();
        for i in 0..64 {
            let x = i % 8;
            let y = i / 8;
            let color = if y < 2 {
                PieceColor::White
            } else if y > 5 {
                PieceColor::Black
            } else {
                PieceColor::None
            };
            let kind = match y {
                1 | 6 => PieceKind::Pawn,
                0 | 7 => match x {
                    0 | 7 => PieceKind::Rook,
                    1 | 6 => PieceKind::Knight,
                    2 | 5 => PieceKind::Bishop,
                    3 => PieceKind::Queen,
                    4 => PieceKind::King,
                    _ => PieceKind::None,
                },
                _ => PieceKind::None,
            };
            pieces.push(Piece { kind, color });
        }

        let mut castles: Vec<Move> = Vec::new();
        castles.push(Move::from(Position::new(4, 0), Position::new(0, 0)));
        castles.push(Move::from(Position::new(4, 0), Position::new(7, 0)));
        castles.push(Move::from(Position::new(4, 7), Position::new(0, 7)));
        castles.push(Move::from(Position::new(4, 7), Position::new(7, 7)));

        Board {
            pieces,
            turn: PieceColor::White,
            selected_piece: None,
            game_state: GameState::Playing,
            white_castle_kingside: true,
            white_castle_queenside: true,
            black_castle_kingside: true,
            black_castle_queenside: true,
            en_passant: None,
            bot: PieceColor::Black,
        }
    }

    pub fn get_piece(&self, position: &Position) -> Piece {
        self.pieces[position.to_index() as usize]
    }

    pub fn set_piece(&mut self, position: &Position, piece: Piece) -> Piece {
        let old_piece = self.pieces[position.to_index() as usize];
        self.pieces[position.to_index() as usize] = piece;
        old_piece
    }

    pub fn remove_piece(&mut self, position: &Position) -> Piece {
        self.set_piece(
            position,
            Piece {
                kind: PieceKind::None,
                color: PieceColor::None,
            },
        )
    }

    pub fn move_piece(&mut self, from: &Position, to: &Position) -> Piece {
        let piece = self.remove_piece(from);
        self.set_piece(to, piece)
    }
}

#[async_trait]
pub trait Engine {
    fn new() -> Self;
    fn get_moves(&self, position: &Position, board: &Board) -> Vec<Position>;
    fn bot_move(&self, window: &Window, bot: &dyn ChessBot, board: &Board) -> Move;

    fn validate_move(&self, mov: &Move, board: &Board) -> Option<Move>;
    fn check_in_check(&self, color: PieceColor, board: &Board) -> bool;
    fn get_game_state(&self, board: &Board) -> GameState;
    fn check_castle(&self, board: &Board, king: Position, rook: Position) -> bool;

    fn execute_move(&self, mov: &Move, board: &Board) -> Option<Board>;
}
