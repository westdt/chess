use std::fmt::{Display, Formatter};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use ts_rs::TS;

use self::{alpha::EngineAlpha, minimax::Minimax};

pub mod alpha;
pub mod minimax;

#[derive(TS, Clone, Serialize, Deserialize, PartialEq)]
#[ts(export, export_to = "../src/bindings/GameState.ts")]
pub enum MoveKind {
    Move,
    Capture,
    EnPassant,
    Dash,
    Castle,
    Promotion,
}

#[derive(TS, Clone, Serialize, Deserialize, PartialEq)]
#[ts(export, export_to = "../src/bindings/GameState.ts")]
pub enum GameState {
    Playing,
    WhiteInCheck,
    BlackInCheck,
    WhiteInCheckmate,
    BlackInCheckmate,
    Stalemate,
    Other,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            GameState::Playing => "Playing",
            GameState::WhiteInCheck => "WhiteInCheck",
            GameState::BlackInCheck => "BlackInCheck",
            GameState::WhiteInCheckmate => "WhiteInCheckmate",
            GameState::BlackInCheckmate => "BlackInCheckmate",
            GameState::Stalemate => "Stalemate",
            GameState::Other => "Other",
        };
        write!(f, "{}", string)
    }
}

#[derive(TS, Clone, Copy, Serialize, Deserialize)]
#[ts(export, export_to = "../src/bindings/PieceColor.ts")]
pub enum PieceColor {
    White,
    Black,
    None,
    Both,
}

impl PartialEq for PieceColor {
    fn eq(&self, other: &Self) -> bool {
        match self {
            PieceColor::White => match other {
                PieceColor::White => true,
                _ => false,
            },
            PieceColor::Black => match other {
                PieceColor::Black => true,
                _ => false,
            },
            PieceColor::None => match other {
                PieceColor::None => true,
                _ => false,
            },
            PieceColor::Both => match other {
                PieceColor::None => false,
                _ => true,
            },
        }
    }
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

impl Display for PieceColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            PieceColor::White => "White",
            PieceColor::Black => "Black",
            PieceColor::None => "None",
            PieceColor::Both => "Both",
        };
        write!(f, "{}", string)
    }
}

#[derive(TS, Clone, Copy, Serialize, Deserialize, PartialEq)]
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

impl Display for PieceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            PieceKind::Pawn => "Pawn",
            PieceKind::Knight => "Knight",
            PieceKind::Bishop => "Bishop",
            PieceKind::Rook => "Rook",
            PieceKind::Queen => "Queen",
            PieceKind::King => "King",
            PieceKind::None => "None",
        };
        write!(f, "{}", string)
    }
}

#[derive(TS, Clone, Copy, Serialize, Deserialize)]
#[ts(export, export_to = "../src/bindings/Piece.ts")]
pub struct Piece {
    pub kind: PieceKind,
    pub color: PieceColor,
}

impl Piece {
    pub fn short_name(&self) -> String {
        let kind = match self.kind {
            PieceKind::Pawn => "P",
            PieceKind::Knight => "N",
            PieceKind::Bishop => "B",
            PieceKind::Rook => "R",
            PieceKind::Queen => "Q",
            PieceKind::King => "K",
            PieceKind::None => " ",
        };

        let color = match self.color {
            PieceColor::White => "w",
            PieceColor::Black => "b",
            PieceColor::None => " ",
            PieceColor::Both => " ",
        };

        format!("{}{}", color, kind)
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.color, self.kind)
    }
}

#[derive(TS, Clone, Copy, Serialize, Deserialize)]
#[ts(export, export_to = "../src/bindings/Position.ts")]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub fn from_index(index: i8) -> Position {
        Position {
            x: index % 8,
            y: index / 8,
        }
    }

    pub fn to_index(&self) -> i8 {
        self.x + self.y * 8
    }

    pub fn from_x_y(x: i8, y: i8) -> Position {
        Position { x, y }
    }

    pub fn to_x_y(&self) -> (i8, i8) {
        (self.x, self.y)
    }

    pub fn from_algebraic(algebraic: String) -> Position {
        let mut chars = algebraic.chars();
        let x = match chars.next().unwrap() {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => -1,
        };
        let y = match chars.next().unwrap() {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => -1,
        };
        Position { x, y }
    }

    pub fn to_algebraic(&self) -> String {
        let x = match self.x {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => '.',
        };
        let y = match self.y {
            0 => '1',
            1 => '2',
            2 => '3',
            3 => '4',
            4 => '5',
            5 => '6',
            6 => '7',
            7 => '8',
            _ => '.',
        };
        format!("{}{}", x, y)
    }

    pub fn add_delta(&self, delta: (i8, i8)) -> Position {
        Position {
            x: self.x + delta.0,
            y: self.y + delta.1,
        }
    }

    pub fn get_delta(&self, other: &Position) -> (i8, i8) {
        (other.x - self.x, other.y - self.y)
    }

    pub fn validate_bounds(&self) -> bool {
        if self.x >= 0 && self.x < 8 && self.y >= 0 && self.y < 8 {
            return true;
        }
        false
    }

    pub fn compare(&self, other: &Position) -> bool {
        if self.x == other.x && self.y == other.y {
            return true;
        }
        false
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[derive(TS, Clone, Serialize, Deserialize)]
#[ts(export, export_to = "../src/bindings/Board.ts")]
pub struct Board {
    pub pieces: Vec<Piece>,
    pub selected: Option<Position>,
    pub en_passant: Option<Position>,
    // (white kingside, white queenside, black kingside, black queenside)
    pub castles: (bool, bool, bool, bool),
    pub turn: PieceColor,
    pub engine: EngineAlpha,
    pub bot_color: PieceColor,
    pub bot: Minimax,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            pieces: vec![
                Piece {
                    kind: PieceKind::None,
                    color: PieceColor::None
                };
                64
            ],
            selected: None,
            en_passant: None,
            castles: (false, false, false, false),
            turn: PieceColor::None,
            engine: EngineAlpha::new(),
            bot_color: PieceColor::None,
            bot: Minimax::new(),
        }
    }

    pub fn default() -> Board {
        let mut pieces: Vec<Piece> = Vec::new();
        for i in 0..64 {
            let piece = match i {
                0 | 7 => Piece {
                    kind: PieceKind::Rook,
                    color: PieceColor::White,
                },
                1 | 6 => Piece {
                    kind: PieceKind::Knight,
                    color: PieceColor::White,
                },
                2 | 5 => Piece {
                    kind: PieceKind::Bishop,
                    color: PieceColor::White,
                },
                3 => Piece {
                    kind: PieceKind::Queen,
                    color: PieceColor::White,
                },
                4 => Piece {
                    kind: PieceKind::King,
                    color: PieceColor::White,
                },
                56 | 63 => Piece {
                    kind: PieceKind::Rook,
                    color: PieceColor::Black,
                },
                57 | 62 => Piece {
                    kind: PieceKind::Knight,
                    color: PieceColor::Black,
                },
                58 | 61 => Piece {
                    kind: PieceKind::Bishop,
                    color: PieceColor::Black,
                },
                59 => Piece {
                    kind: PieceKind::Queen,
                    color: PieceColor::Black,
                },
                60 => Piece {
                    kind: PieceKind::King,
                    color: PieceColor::Black,
                },
                8..=15 => Piece {
                    kind: PieceKind::Pawn,
                    color: PieceColor::White,
                },
                48..=55 => Piece {
                    kind: PieceKind::Pawn,
                    color: PieceColor::Black,
                },
                _ => Piece {
                    kind: PieceKind::None,
                    color: PieceColor::None,
                },
            };
            pieces.push(piece);
        }

        Board {
            pieces,
            selected: None,
            en_passant: None,
            castles: (true, true, true, true),
            turn: PieceColor::White,
            engine: EngineAlpha::new(),
            bot_color: PieceColor::Black,
            bot: Bot::new(),
        }
    }

    pub fn set_piece(&mut self, position: &Position, piece: Piece) -> Piece {
        let old_piece = self.pieces[position.to_index() as usize];
        self.pieces[position.to_index() as usize] = piece;
        old_piece
    }

    pub fn get_piece(&self, position: &Position) -> Piece {
        self.pieces[position.to_index() as usize]
    }

    pub fn move_piece(&mut self, from: &Position, to: &Position) -> Piece {
        let old_piece = self.get_piece(from);
        let piece = self.get_piece(from);
        self.set_piece(
            from,
            Piece {
                kind: PieceKind::None,
                color: PieceColor::None,
            },
        );
        self.set_piece(to, piece);
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

    pub fn get_moves(&self, position: Position) -> Vec<Position> {
        self.engine.get_moves(self, position)
    }

    pub fn bot_move(&mut self) {
        let bot_move = self.bot.get_best_move(self);
        self.move_piece(&bot_move.0, &bot_move.1);
        self.turn = self.turn.opposite();
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for y in 0..8 {
            let y = 7 - y;
            for x in 0..8 {
                let piece = self.get_piece(&Position::from_x_y(x, y));
                string.push_str(&format!("{} ", piece.short_name()));
            }
            string.push_str("\n");
        }

        let selected = match self.selected {
            Some(position) => position.to_algebraic(),
            None => String::from("None"),
        };

        let en_passant = match self.en_passant {
            Some(position) => position.to_algebraic(),
            None => String::from("None"),
        };

        write!(f, "board:\n{}selected_piece: {}\nen passant: {}\ncastles: {}\ncurrent turn:{}\nbot color:{}", string, selected, en_passant, format!("{:?}", self.castles), self.turn, self.bot_color)
    }
}

pub trait Engine: Clone + Serialize + DeserializeOwned {
    fn new() -> Self;
    fn get_moves(&self, board: &Board, position: Position) -> Vec<Position>;
    fn validate_move(&self, board: &Board, from: Position, to: Position) -> bool;
    fn execute_move(&self, board: &mut Board, from: Position, to: Position) -> MoveKind;
    fn get_move_kind(&self, board: &Board, from: Position, to: Position) -> MoveKind;
    fn get_game_state(&self, board: &Board) -> GameState;
}

pub trait Bot: Clone + Serialize + DeserializeOwned {
    fn new() -> Self;
    fn get_best_move(&self, board: &Board) -> (Position, Position);
}
