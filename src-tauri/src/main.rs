// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env,
    fmt::{self, Display}, i16::{MIN, MAX}, cmp::{max, min},
};

use log::LevelFilter;
use serde::{de, Deserialize, Serialize};
use tauri::{Manager, Window};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use ts_rs::TS;

mod utils;

static mut BOARD: Board = Board {
    pieces: [Piece {
        color: PieceColor::None,
        kind: PieceType::None,
    }; 64],
    en_passant: None,
    turn: PieceColor::White,
    valid_castles: vec![],
    selected_piece: None,
};

#[derive(TS)]
#[ts(export, export_to = "../src/bindings/GameState.ts")]
enum GameState {
    Playing,
    WhiteCheck,
    BlackCheck,
    WhiteCheckmate,
    BlackCheckmate,
    Stalemate,
    Error,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			GameState::Playing => write!(f, "Playing"),
			GameState::WhiteCheck => write!(f, "WhiteCheck"),
			GameState::BlackCheck => write!(f, "BlackCheck"),
			GameState::WhiteCheckmate => write!(f, "WhiteCheckmate"),
			GameState::BlackCheckmate => write!(f, "BlackCheckmate"),
			GameState::Stalemate => write!(f, "Stalemate"),
			GameState::Error => write!(f, "Error"),
		}
	}
}

#[derive(TS)]
#[ts(export, export_to = "../src/bindings/PieceColor.ts")]
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
enum PieceColor {
    White,
    Black,
    None,
}

impl fmt::Display for PieceColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PieceColor::White => write!(f, "White"),
            PieceColor::Black => write!(f, "Black"),
            PieceColor::None => write!(f, "None"),
        }
    }
}

impl PieceColor {
    fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
            PieceColor::None => PieceColor::None,
        }
    }
}

#[derive(TS)]
#[ts(export, export_to = "../src/bindings/PieceType.ts")]
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None,
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PieceType::Pawn => write!(f, "Pawn"),
            PieceType::Knight => write!(f, "Knight"),
            PieceType::Bishop => write!(f, "Bishop"),
            PieceType::Rook => write!(f, "Rook"),
            PieceType::Queen => write!(f, "Queen"),
            PieceType::King => write!(f, "King"),
            PieceType::None => write!(f, "None"),
        }
    }
}

#[derive(Clone, Copy)]
struct Piece {
    color: PieceColor,
    kind: PieceType,
}

#[derive(Clone)]
struct Board {
    pieces: [Piece; 64],
    en_passant: Option<Location>,
    turn: PieceColor,
    valid_castles: Vec<(Location, Location)>,
    selected_piece: Option<Location>,
}

impl Board {
    fn set_piece(&mut self, location: &Location, piece: &Piece) {
        trace!(
            "set_piece() {} {} {}.",
            location.to_algebraic(),
            piece.color,
            piece.kind
        );
        self.pieces[location.value as usize] = piece.clone();
    }

    fn get_piece(&self, location: &Location) -> Piece {
		if !self.check_bounds(location) {
			return Piece {
				color: PieceColor::None,
				kind: PieceType::None,
			};
		}

        self.pieces[location.value as usize].clone()
    }

    fn get_moves(&self, location: &Location, validate_check: bool) -> Vec<Location> {
        // debug!("get_moves() was called.");
        let piece = self.get_piece(location);

        match piece.kind {
            PieceType::Pawn => get_moves_pawn(self, location, validate_check),
            PieceType::Knight => get_moves_knight(self, location, validate_check),
            PieceType::Bishop => get_moves_bishop(self, location, validate_check),
            PieceType::Rook => get_moves_rook(self, location, validate_check),
            PieceType::Queen => get_moves_queen(self, location, validate_check),
            PieceType::King => get_moves_king(self, location, validate_check),
            PieceType::None => Vec::new(),
        }
    }

    fn check_bounds(&self, location: &Location) -> bool {
        location.value >= 0 && location.value < 64
    }

    fn check_bounds_x_y(&self, x: i16, y: i16) -> bool {
        x >= 0 && x < 8 && y >= 0 && y < 8
    }

    fn validate_move(
        &self,
        from: (i16, i16),
        to: (i16, i16),
        validate_check: bool,
    ) -> Option<Location> {
        let bound_check = self.check_bounds_x_y(to.0, to.1);
        if !bound_check {
            return None;
        }

        let from_piece = self.get_piece(&Location::from_x_y(from.0, from.1));
        let to_piece = self.get_piece(&Location::from_x_y(to.0, to.1));

        if from_piece.color == to_piece.color {
            return None;
        }

        if validate_check {
            let mut board = self.clone();
            board.set_piece(&Location::from_x_y(to.0, to.1), &from_piece);
            board.set_piece(
                &Location::from_x_y(from.0, from.1),
                &Piece {
                    color: PieceColor::None,
                    kind: PieceType::None,
                },
            );

            if board.get_in_check(from_piece.color) {
                return None;
            }
        }

        Some(Location::from_x_y(to.0, to.1))
    }

    fn get_in_check(&self, color: PieceColor) -> bool {
        let mut king_location: Option<Location> = None;
        for i in 0..64 {
            let piece = self.get_piece(&Location::new(i as i16));
            if piece.color == color && piece.kind == PieceType::King {
                king_location = Some(Location::new(i as i16));
                break;
            }
        }

        let king_location = match king_location {
            Some(king_location) => king_location,
            None => return false,
        };

        for i in 0..64 {
            let piece = self.get_piece(&Location::new(i as i16));
            if piece.color == color.opposite() {
                let moves = self.get_moves(&Location::new(i as i16), false);
                for j in 0..moves.len() {
                    if moves[j].compare(&king_location) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn get_move_notation(from: &Location, to: &Location) -> String {
        let result = "Unknown".to_string();

        result
    }

    fn execute_move(&mut self, from: &Location, to: &Location) {
        let piece = self.get_piece(from);
        self.set_piece(to, &piece);
        self.set_piece(
            from,
            &Piece {
                color: PieceColor::None,
                kind: PieceType::None,
            },
        );

        self.en_passant = None;
        if piece.kind == PieceType::Pawn {
			if (from.to_x_y().1 - to.to_x_y().1 as i16).abs() > 2 {

			} else {
				if (from.to_x_y().1 - to.to_x_y().1 as i16).abs() == 2 {
					self.en_passant = Some(Location::from_x_y(
						to.to_x_y().0,
						(from.to_x_y().1 + to.to_x_y().1) / 2,
					));
				} else if to.to_x_y().1 == 0 || to.to_x_y().1 == 7 {
					info!("Promotion");
					self.set_piece(
						to,
						&Piece {
							color: piece.color,
							kind: PieceType::Queen,
						},
					);
				}
			}
        } else if piece.kind == PieceType::King {
            if from.to_x_y().0 == 4 {
                let mut i = 0;
                while i < self.valid_castles.len() {
                    if self.valid_castles[i as usize].0.compare(from) {
                        self.valid_castles.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
        } else if piece.kind == PieceType::Rook {
            if (from.to_x_y().0 == 0 || from.to_x_y().0 == 7) {
                let mut i = 0;
                while i < self.valid_castles.len() {
                    if self.valid_castles[i].1.compare(from) {
                        self.valid_castles.remove(i);
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }

    fn execute_move_and_get_requests(&mut self, from: &Location, to: &Location) -> Vec<JsRequest> {
        let piece_from = self.get_piece(from);
        self.execute_move(from, to);
        let piece_to = self.get_piece(to);

        let mut requests: Vec<JsRequest> = Vec::new();

        if piece_from.kind != piece_to.kind {
			// promotion
			requests.push(JsRequest {
				jsfunction: JsFunction::PromotePiece,
				args: vec![
					from.to_algebraic(),
					to.to_algebraic(), 
					piece_to.color.to_string(),
					piece_to.kind.to_string()
					],
			});
        } else {
            requests.push(JsRequest {
                jsfunction: JsFunction::MovePiece,
                args: vec![from.to_algebraic(), to.to_algebraic()],
            });
        }

        requests
    }

    fn get_game_state(&self) -> GameState {
        let mut white_has_moves = false;
        let mut black_has_moves = false;

        for i in 0..64 {
            match self.get_piece(&Location { value: i }).color {
                PieceColor::White => {
                    if (!white_has_moves) {
                        if self.get_moves(&Location { value: i }, true).len() > 0 {
                            white_has_moves = true;
                        };
                    }
                }
                PieceColor::Black => {
                    if (!black_has_moves) {
                        if self.get_moves(&Location { value: i }, true).len() > 0 {
                            black_has_moves = true;
                        };
                    }
                }
                PieceColor::None => {}
            }
        }

        if black_has_moves || white_has_moves {
            if black_has_moves && white_has_moves {
                if self.get_in_check(PieceColor::White) {
                    info!("White is in check");
                    GameState::WhiteCheck
                } else if self.get_in_check(PieceColor::Black) {
                    info!("Black is in check");
                    GameState::BlackCheck
                } else {
                    GameState::Playing
                }
            } else if black_has_moves {
                if self.get_in_check(PieceColor::White) {
                    info!("Black wins by checkmate");
                    GameState::WhiteCheckmate
                } else {
                    info!("Stalemate");
                    GameState::Stalemate
                }
            } else {
                if self.get_in_check(PieceColor::Black) {
                    info!("White wins by checkmate");
                    GameState::BlackCheckmate
                } else {
                    info!("Stalemate");
                    GameState::Stalemate
                }
            }
        } else {
            info!("Stalemate");
            GameState::Stalemate
        }
    }
}

fn get_moves_pawn(board: &Board, location: &Location, validate_check: bool) -> Vec<Location> {
    let mut moves: Vec<Location> = Vec::new();
    trace!(
        "get_moves_pawn() was called on {}.",
        location.to_algebraic()
    );
    let (x, y) = location.to_x_y();

    let direction = match board.get_piece(location).color {
        PieceColor::White => 1,
        PieceColor::Black => -1,
        PieceColor::None => 0,
    };

    if direction == 0 {
        return moves;
    }

    match board.validate_move((x, y), (x, y + direction), validate_check) {
        Some(location) => {
            if board.get_piece(&location).color == PieceColor::None {
                moves.push(location);
            }

            if direction == 1 && y == 1 || direction == -1 && y == 6 {
                match board.validate_move((x, y), (x, y + direction * 2), validate_check) {
                    Some(location) => {
                        if board.get_piece(&location).color == PieceColor::None {
                            moves.push(location);
                        }
                    }
                    None => {}
                }
            }
        }
        None => {}
    }

    for i in -1..2 {
        if i == 0 {
            continue;
        }

        if board
            .get_piece(&Location::from_x_y(x + i, y + direction))
            .color
            == board.get_piece(location).color.opposite()
        {
            match board.validate_move((x, y), (x + i, y + direction), validate_check) {
                Some(location) => moves.push(location),
                None => {}
            }
        } else {
            match (board.en_passant) {
                Some(en_passant) => {
                    if en_passant.compare(&Location::from_x_y(x + i, y + direction)) {
                        match board.validate_move((x, y), (x + i, y + direction), validate_check) {
                            Some(location) => moves.push(location),
                            None => {}
                        }
                    }
                }
                None => {}
            }
        }
    }
    moves
}

fn get_moves_knight(board: &Board, location: &Location, validate_check: bool) -> Vec<Location> {
    let mut moves: Vec<Location> = Vec::new();
    trace!(
        "get_moves_knight() was called on {}.",
        location.to_algebraic()
    );
    let (x, y) = location.to_x_y();
    for i in -2..3 {
        for j in -2..3 {
            if (i as i16).abs() + (j as i16).abs() == 3 {
                match board.validate_move((x, y), (x + i, y + j), validate_check) {
                    Some(location) => moves.push(location),
                    None => {}
                }
            }
        }
    }
    moves
}

fn get_moves_bishop(board: &Board, location: &Location, validate_check: bool) -> Vec<Location> {
    let steps = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    get_moves_scanline(board, location, &steps, validate_check)
}

fn get_moves_rook(board: &Board, location: &Location, validate_check: bool) -> Vec<Location> {
    let steps = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    get_moves_scanline(board, location, &steps, validate_check)
}

fn get_moves_queen(board: &Board, location: &Location, validate_check: bool) -> Vec<Location> {
    let steps = [
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
    ];
    get_moves_scanline(board, location, &steps, validate_check)
}

fn get_moves_scanline(
    board: &Board,
    location: &Location,
    steps: &[(i16, i16)],
    validate_check: bool,
) -> Vec<Location> {
    let mut moves: Vec<Location> = Vec::new();
    trace!(
        "get_moves_queen() was called on {}.",
        location.to_algebraic()
    );
    let (x, y) = location.to_x_y();

    for step in steps {
        let mut i = 1;
        loop {
            let new_x = x + step.0 * i;
            let new_y = y + step.1 * i;
            if !board.check_bounds_x_y(new_x, new_y) {
                break;
            }

            if board.get_piece(location).color
                == board.get_piece(&Location::from_x_y(new_x, new_y)).color
            {
                break;
            }

            match board.validate_move((x, y), (new_x, new_y), validate_check) {
                Some(location) => {
                    moves.push(location);
                    match board.get_piece(&location).color {
                        PieceColor::None => {}
                        _ => break,
                    }
                }
                None => {}
            }
            i += 1;
        }
    }

    moves
}

fn get_moves_king(board: &Board, location: &Location, validate_check: bool) -> Vec<Location> {
    let mut moves: Vec<Location> = Vec::new();
    trace!(
        "get_moves_king() was called on {}.",
        location.to_algebraic()
    );
    let (x, y) = location.to_x_y();

    let steps = [
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
    ];
    for step in steps {
        let new_x = x + step.0;
        let new_y = y + step.1;
        if !board.check_bounds_x_y(new_x, new_y) {
            continue;
        }

        match board.validate_move((x, y), (new_x, new_y), validate_check) {
            Some(location) => moves.push(location),
            None => {}
        }
    }

    // castling
    for valid_castle in &board.valid_castles {
        if valid_castle.0.compare(location) {
            match valid_castle.1.to_x_y().0 {
                7 => {
                    let mut valid = true;
                    for i in 1..3 {
                        match board.validate_move((x, y), (x + i, y), validate_check) {
                            Some(location) => match board.get_piece(&location).color {
                                PieceColor::None => {}
                                _ => {
                                    valid = false;
                                    break;
                                }
                            },
                            None => {
                                valid = false;
                                break;
                            }
                        }
                    }

                    if valid {
                        moves.push(valid_castle.1);
                    }
                }
                0 => {
                    let mut valid = true;
                    for i in 1..4 {
                        match board.validate_move((x, y), (x - i, y), validate_check) {
                            Some(location) => match board.get_piece(&location).color {
                                PieceColor::None => {}
                                _ => {
                                    valid = false;
                                    break;
                                }
                            },
                            None => {
                                valid = false;
                                break;
                            }
                        }
                    }

                    if valid {
                        moves.push(valid_castle.1);
                    }
                }
                _ => {}
            }
        }
    }

    moves
}

#[derive(Clone, Copy)]
struct Location {
    // location struct where a1 is 0, a8 is 7, h1 is 56, h8 is 63, etc
    value: i16,
}

impl Location {
    fn new(value: i16) -> Location {
        Location { value }
    }

    fn to_algebraic(&self) -> String {
        let mut file = 'a';
        let mut rank = '1';
        match self.value % 8 {
            0 => file = 'a',
            1 => file = 'b',
            2 => file = 'c',
            3 => file = 'd',
            4 => file = 'e',
            5 => file = 'f',
            6 => file = 'g',
            7 => file = 'h',
            _ => {}
        }
        match self.value / 8 {
            0 => rank = '1',
            1 => rank = '2',
            2 => rank = '3',
            3 => rank = '4',
            4 => rank = '5',
            5 => rank = '6',
            6 => rank = '7',
            7 => rank = '8',
            _ => {}
        }
        format!("{}{}", file, rank)
    }

    fn from_algebraic(algebraic: &str) -> Location {
        let mut file = 0;
        let mut rank = 0;
        match algebraic.chars().nth(0).unwrap() {
            'a' => file = 0,
            'b' => file = 1,
            'c' => file = 2,
            'd' => file = 3,
            'e' => file = 4,
            'f' => file = 5,
            'g' => file = 6,
            'h' => file = 7,
            _ => {}
        }
        match algebraic.chars().nth(1).unwrap() {
            '1' => rank = 0,
            '2' => rank = 1,
            '3' => rank = 2,
            '4' => rank = 3,
            '5' => rank = 4,
            '6' => rank = 5,
            '7' => rank = 6,
            '8' => rank = 7,
            _ => {}
        }
        Location::new(rank * 8 + file)
    }

    fn to_x_y(&self) -> (i16, i16) {
        (self.value % 8, self.value / 8)
    }

    fn from_x_y(x: i16, y: i16) -> Location {
        Location::new(y * 8 + x)
    }

    fn compare(&self, other: &Location) -> bool {
        self.value == other.value
    }
}

#[derive(TS)]
#[ts(export, export_to = "../src/bindings/JsFunction.ts")]
#[derive(Serialize, Deserialize)]
enum JsFunction {
    None,

    CreatePiece,
    MovePiece,
    DeletePiece,
	PromotePiece,
    DeleteBoard,

    CreateHighlight,
    DeleteHighlights,

	CallChessBot,
	GameOver
}

#[derive(Serialize)]
struct JsRequest {
    jsfunction: JsFunction,
    args: Vec<String>,
}

fn create_blank_board() -> Board {
    let blank_board = Board {
        pieces: [Piece {
            color: PieceColor::None,
            kind: PieceType::None,
        }; 64],
        en_passant: None,
        turn: PieceColor::White,
        valid_castles: vec![],
        selected_piece: None,
    };
    blank_board
}

fn generate_new_board() -> Board {
    let mut board = create_blank_board();

    for i in 0..8 {
        board.set_piece(
            &Location::new(i + 8 * 1),
            &Piece {
                color: PieceColor::White,
                kind: PieceType::Pawn,
            },
        );
        board.set_piece(
            &Location::new(i + 8 * 6),
            &Piece {
                color: PieceColor::Black,
                kind: PieceType::Pawn,
            },
        );
    }

    board.set_piece(
        &Location::new(0),
        &Piece {
            color: PieceColor::White,
            kind: PieceType::Rook,
        },
    );
    board.set_piece(
        &Location::new(1),
        &Piece {
            color: PieceColor::White,
            kind: PieceType::Knight,
        },
    );
    board.set_piece(
        &Location::new(2),
        &Piece {
            color: PieceColor::White,
            kind: PieceType::Bishop,
        },
    );
    board.set_piece(
        &Location::new(3),
        &Piece {
            color: PieceColor::White,
            kind: PieceType::Queen,
        },
    );
    board.set_piece(
        &Location::new(4),
        &Piece {
            color: PieceColor::White,
            kind: PieceType::King,
        },
    );
    board.set_piece(
        &Location::new(5),
        &Piece {
            color: PieceColor::White,
            kind: PieceType::Bishop,
        },
    );
    board.set_piece(
        &Location::new(6),
        &Piece {
            color: PieceColor::White,
            kind: PieceType::Knight,
        },
    );
    board.set_piece(
        &Location::new(7),
        &Piece {
            color: PieceColor::White,
            kind: PieceType::Rook,
        },
    );

    board.set_piece(
        &Location::new(0 + 8 * 7),
        &Piece {
            color: PieceColor::Black,
            kind: PieceType::Rook,
        },
    );
    board.set_piece(
        &Location::new(1 + 8 * 7),
        &Piece {
            color: PieceColor::Black,
            kind: PieceType::Knight,
        },
    );
    board.set_piece(
        &Location::new(2 + 8 * 7),
        &Piece {
            color: PieceColor::Black,
            kind: PieceType::Bishop,
        },
    );
    board.set_piece(
        &Location::new(3 + 8 * 7),
        &Piece {
            color: PieceColor::Black,
            kind: PieceType::Queen,
        },
    );
    board.set_piece(
        &Location::new(4 + 8 * 7),
        &Piece {
            color: PieceColor::Black,
            kind: PieceType::King,
        },
    );
    board.set_piece(
        &Location::new(5 + 8 * 7),
        &Piece {
            color: PieceColor::Black,
            kind: PieceType::Bishop,
        },
    );
    board.set_piece(
        &Location::new(6 + 8 * 7),
        &Piece {
            color: PieceColor::Black,
            kind: PieceType::Knight,
        },
    );
    board.set_piece(
        &Location::new(7 + 8 * 7),
        &Piece {
            color: PieceColor::Black,
            kind: PieceType::Rook,
        },
    );

    board
        .valid_castles
        .push((Location::new(4), Location::new(7)));
    board
        .valid_castles
        .push((Location::new(4), Location::new(0)));
    board
        .valid_castles
        .push((Location::new(60), Location::new(63)));
    board
        .valid_castles
        .push((Location::new(60), Location::new(56)));

    board
}

#[tauri::command]
fn select(location: String) -> Vec<JsRequest> {
    let mut board = unsafe { BOARD.clone() };
    debug!("turn is {}.", board.turn);
    //info!("select() was called.");

    let mut requests: Vec<JsRequest> = Vec::new();
    let from = board.selected_piece;
    let to = Location::from_algebraic(&location);

    requests.push(JsRequest {
        jsfunction: JsFunction::DeleteHighlights,
        args: vec![],
    });

    match from {
        Some(from) => {
            if from.compare(&to) {
                board.selected_piece = None;
            } else {
                let source = board.get_piece(&from);
                let destination = board.get_piece(&to);

                if destination.color == source.color {
                    let mut castle = false;
                    if source.kind == PieceType::King {
                        if destination.kind == PieceType::Rook {
                            for valid_castle in &board.clone().valid_castles {
                                if valid_castle.0.compare(&from) && valid_castle.1.compare(&to) {
                                    castle = true;

                                    let king = from;
                                    let rook = valid_castle.1;
                                    let mut new_king_x = 0;
                                    let mut new_rook_x = 0;

                                    match rook.to_x_y().0 {
                                        0 => {
                                            new_king_x = 2;
                                            new_rook_x = 3;
                                        }
                                        7 => {
                                            new_king_x = 6;
                                            new_rook_x = 5;
                                        }
                                        _ => {}
                                    }

                                    let king = Location::from_x_y(new_king_x, king.to_x_y().1);
                                    let rook = Location::from_x_y(new_rook_x, rook.to_x_y().1);

                                    let mut execute1 =
                                        board.execute_move_and_get_requests(&from, &king);
                                    let mut execute2 =
                                        board.execute_move_and_get_requests(&valid_castle.1, &rook);

                                    requests.append(&mut execute1);
                                    requests.append(&mut execute2);

                                    board.selected_piece = None;
                                    board.turn = board.turn.opposite();

                                    break;
                                }
                            }
                        }
                    }

                    board.selected_piece = Some(to);

                    let moves = board.get_moves(&to, true);
                    for i in 0..moves.len() {
                        requests.push(JsRequest {
                            jsfunction: JsFunction::CreateHighlight,
                            args: vec!["0".to_string(), moves[i].to_algebraic()],
                        });
                    }

                    requests.push(JsRequest {
                        jsfunction: JsFunction::CreateHighlight,
                        args: vec!["1".to_string(), to.to_algebraic()],
                    });
                } else if destination.color == source.color.opposite() {
                    let moves = board.get_moves(&from, true);
                    let mut valid = false;

                    for i in 0..moves.len() {
                        if moves[i].compare(&to) {
                            valid = true;
                            break;
                        }
                    }

                    if !valid {
                        board.selected_piece = None;
                        return requests;
                    }

                    let mut execute = board.execute_move_and_get_requests(&from, &to);
                    requests.append(&mut execute);

                    board.selected_piece = None;
                    board.turn = board.turn.opposite();
                } else {
					board.selected_piece = None;
                    let moves = board.get_moves(&from, true);
                    let mut valid = false;
                    for i in 0..moves.len() {
                        if moves[i].compare(&to) {
                            valid = true;
                            break;
                        }
                    }

                    if valid {
                        let mut execute = board.execute_move_and_get_requests(&from, &to);
                    	requests.append(&mut execute);

                		board.selected_piece = None;
                    	board.turn = board.turn.opposite();
                    }
                }
            }
        }
        None => {
            if board.get_piece(&to).color == (board.turn) {
                board.selected_piece = Some(to);

                let moves = board.get_moves(&to, true);
                for i in 0..moves.len() {
                    requests.push(JsRequest {
                        jsfunction: JsFunction::CreateHighlight,
                        args: vec!["0".to_string(), moves[i].to_algebraic()],
                    });
                }

                requests.push(JsRequest {
                    jsfunction: JsFunction::CreateHighlight,
                    args: vec!["1".to_string(), to.to_algebraic()],
                });
            }
        }
    }

    let mut game_state_requests = get_game_state_request(&board);
	requests.append(&mut game_state_requests);

	if board.turn == PieceColor::Black {
		requests.push(JsRequest {
			jsfunction: JsFunction::CallChessBot,
			args: vec![],
		});
	}

    unsafe {
        BOARD = board;
    }

    requests
}

fn get_game_state_request(board: &Board) -> Vec<JsRequest> {
	let mut requests = Vec::new();
	let game_state = board.get_game_state();

	let mut send_request = false;

	match game_state {
		GameState::Playing => {},
		GameState::Error => {},
		GameState::WhiteCheck => {},
		GameState::BlackCheck => {},
		_ => {
			send_request = true;
		}
	}

	if send_request {
		requests.push(JsRequest {
			jsfunction: JsFunction::GameOver,
			args: vec![game_state.to_string()],
		});
	}

	requests
}


#[tauri::command]
fn setup() -> Vec<JsRequest> {
    info!("setup() was called.");

    let mut requests: Vec<JsRequest> = Vec::new();

    let board = generate_new_board();
    for i in 0..64 {
        if board.pieces[i].kind == PieceType::None {
            continue;
        }

        requests.push(JsRequest {
            jsfunction: JsFunction::CreatePiece,
            args: vec![
                board.pieces[i].color.to_string(),
                board.pieces[i].kind.to_string(),
                Location::new(i as i16).to_algebraic(),
            ],
        });
    }

    unsafe {
        BOARD = board;
    }

    requests
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .with_colors(ColoredLevelConfig::default())
                .level(LevelFilter::Debug)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            setup,
            select,
			chess_bot,
            utils::js_error,
            utils::js_warn,
            utils::js_info,
            utils::js_debug,
            utils::js_trace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

const DEPTH: usize = 4;

#[tauri::command]
fn chess_bot() -> Vec<JsRequest> {
	let mut board = unsafe { BOARD.clone() };
	let mut requests = Vec::new();

	let mut best_score = MIN;
	let mut best_move: (Location, Location) = (Location::new(0), Location::new(0));

	for i in 0..64 {
		let piece = board.get_piece(&Location::new(i as i16));
		if piece.color != board.turn {
			continue;
		}

		let moves = board.get_moves(&Location::new(i as i16), true);
		for j in 0..moves.len() {
			let mut new_board = board.clone();
			new_board.execute_move(&Location::new(i as i16), &moves[j]);
			let score = layer(new_board, DEPTH, MIN, MAX);

			if score >= best_score {
				debug!("{} {} {}", Location::new(i as i16).to_algebraic(), moves[j].to_algebraic(), score);
				best_score = score;
				best_move = (Location::new(i as i16), moves[j]);
			}
		}
	}

	let mut execute = board.execute_move_and_get_requests(&best_move.0, &best_move.1);
	requests.append(&mut execute);

	board.turn = board.turn.opposite();

	let mut game_state_requests = get_game_state_request(&board);
	requests.append(&mut game_state_requests);

	unsafe {
		BOARD = board;
	}

	requests
}

fn evaluate(board: &Board) -> i16 {
	let mut score = 0;

	for i in 0..64 {
		let piece = board.get_piece(&Location::new(i as i16));
		if piece.color == board.turn.opposite() {
			match piece.kind {
				PieceType::Pawn => score -= 100,
				PieceType::Knight => score -= 320,
				PieceType::Bishop => score -= 330,
				PieceType::Rook => score -= 500,
				PieceType::Queen => score -= 900,
				PieceType::King => score -= 20000,
				PieceType::None => {}
			}
		} else if piece.color == board.turn {
			match piece.kind {
				PieceType::Pawn => score += 100,
				PieceType::Knight => score += 320,
				PieceType::Bishop => score += 330,
				PieceType::Rook => score += 500,
				PieceType::Queen => score += 900,
				PieceType::King => score += 20000,
				PieceType::None => {}
			}
		}
	}

	score
}

fn layer(board: Board, depth: usize, mut alpha: i16, mut beta: i16) -> i16 {
	let mut a = alpha;
	let mut b = beta;
	if depth == 0 {
		return (evaluate(&board));
	}

	let mut score = 0;
	if board.turn == PieceColor::Black {
		score = MIN;
		for i in 0..64 {
			let piece = board.get_piece(&Location::new(i as i16));
			if piece.color != board.turn {
				continue;
			}

			let moves = board.get_moves(&Location::new(i as i16), true);
			for j in 0..moves.len() {
				let mut new_board = board.clone();
				new_board.execute_move(&Location::new(i as i16), &moves[j]);
				new_board.turn = new_board.turn.opposite();

				score = max(score, layer(new_board, depth - 1, a, b));

				if score >= b {
					break;
				}

				a = max(a, score);
			}
		}
	} else {
		score = MAX;
		for i in 0..64 {
			let piece = board.get_piece(&Location::new(i as i16));
			if piece.color != board.turn {
				continue;
			}

			let moves = board.get_moves(&Location::new(i as i16), true);
			for j in 0..moves.len() {
				let mut new_board = board.clone();
				new_board.execute_move(&Location::new(i as i16), &moves[j]);
				new_board.turn = new_board.turn.opposite();

				score = min(score, layer(new_board, depth - 1, a, b));

				if score <= alpha {
					break;
				}

				b = min(b, score);
			}
		}
	}

	score
}