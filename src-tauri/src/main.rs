// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, fmt};

use log::LevelFilter;
use serde::{Deserialize, Serialize};
use tauri::Window;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use utils::{from_algebraic, from_algebraic_str, to_algebraic};

mod utils;

static mut BOARD: [Piece; 64] = [Piece {
    piece_color: PieceColor::None,
    piece_kind: PieceKind::None,
}; 64];
static mut EN_PASSANT: Option<Location> = None;
static mut TURN: PieceColor = PieceColor::White;
static mut SELECTED_PIECE: Option<Location> = None;
static mut VALID_CASTLES: Vec<(Location, Location)> = vec![];

#[derive(Clone, Copy, PartialEq)]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None,
}

impl fmt::Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PieceKind::Pawn => write!(f, "Pawn"),
            PieceKind::Knight => write!(f, "Knight"),
            PieceKind::Bishop => write!(f, "Bishop"),
            PieceKind::Rook => write!(f, "Rook"),
            PieceKind::Queen => write!(f, "Queen"),
            PieceKind::King => write!(f, "King"),
            PieceKind::None => write!(f, "None"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PieceColor {
    White,
    Black,
    None,
}

impl fmt::Display for PieceColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PieceColor::White => write!(f, "White"),
            PieceColor::Black => write!(f, "Black"),
            PieceColor::None => write!(f, "None"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    WhiteCheck,
    BlackCheck,
    WhiteCheckmate,
    BlackCheckmate,
    Stalemate,
    Error,
}

#[derive(Serialize, Deserialize, Clone)]
struct JsRequest {
    function: String,
    arguments: Vec<String>,
}

impl JsRequest {
    fn new(function: String) -> JsRequest {
        JsRequest {
            function,
            arguments: Vec::new(),
        }
    }

    fn add_argument(&mut self, argument: String) -> JsRequest {
        let mut request = self.clone();
        request.arguments.push(argument);
        request
    }
}

#[derive(Clone, Copy)]
struct Piece {
    piece_color: PieceColor,
    piece_kind: PieceKind,
}

impl Piece {
    fn new(piece_color: PieceColor, piece_kind: PieceKind) -> Piece {
        Piece {
            piece_color,
            piece_kind,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Location {
    location: usize,
}

impl Location {
    fn new(location: usize) -> Location {
        Location { location }
    }

    fn from_string(algebraic: String) -> Location {
        let (x, y) = from_algebraic(algebraic);
        Location::from_x_y(x, y)
    }

    fn from_str(algebraic: &str) -> Location {
        let (x, y) = from_algebraic_str(algebraic);
        Location::from_x_y(x, y)
    }

    fn from_x_y(x: i16, y: i16) -> Location {
        Location {
            location: (y * 8 + x) as usize,
        }
    }

    fn to_string(&self) -> String {
        let x = (self.location % 8) as i16;
        let y = (self.location / 8) as i16;
        to_algebraic((x, y))
    }

    fn to_x_y(&self) -> (i16, i16) {
        let x = (self.location % 8) as i16;
        let y = (self.location / 8) as i16;
        (x, y)
    }

    fn get_moves(&self, board: [Piece; 64], validate_check: bool) -> Vec<Location> {
        let (x, y) = self.to_x_y();
        let piece = get_piece(board, x, y);

        let vec = match piece.piece_kind {
            PieceKind::Pawn => get_moves_pawn(self, board, validate_check),
            PieceKind::Knight => get_moves_knight(self, board, validate_check),
            PieceKind::Bishop => get_moves_bishop(self, board, validate_check),
            PieceKind::Rook => get_moves_rook(self, board, validate_check),
            PieceKind::Queen => get_moves_queen(self, board, validate_check),
            PieceKind::King => get_moves_king(self, board, validate_check),
            _ => Vec::new(),
        };

        // debug!("Valid moves for piece at {} are {:?}", self.to_string(), vec);

        vec
    }

    fn get_piece(&self, board: [Piece; 64]) -> Piece {
        let (x, y) = self.to_x_y();
        get_piece(board, x, y)
    }

    fn get_color(&self, board: [Piece; 64]) -> PieceColor {
        let (x, y) = self.to_x_y();
        get_piece(board, x, y).piece_color
    }

    fn is_king(&self, board: [Piece; 64]) -> bool {
        let (x, y) = self.to_x_y();
        get_piece(board, x, y).piece_kind == PieceKind::King
    }

    fn check_bounds(x: i16, y: i16) -> Option<Location> {
        if x >= 0 && x <= 7 && y >= 0 && y <= 7 {
            Some(Location::from_x_y(x as i16, y as i16))
        } else {
            None
        }
    }
}

fn set_piece(board: [Piece; 64], piece: Piece, x: i16, y: i16) {
    unsafe {
        BOARD[(y * 8 + x) as usize] = piece;
    }
}

fn get_piece(board: [Piece; 64], x: i16, y: i16) -> Piece {
    board[(y * 8 + x) as usize]
}

fn get_moves_pawn(location: &Location, board: [Piece; 64], validate_check: bool) -> Vec<Location> {
    trace!("Getting moves for pawn at {}", location.to_string());
    let mut valid_moves: Vec<Location> = Vec::new();

    let (x, y) = location.to_x_y();
    let x = x as i16;
    let y = y as i16;

    match location.get_color(board) {
        PieceColor::White => {
            match Location::check_bounds(x, y + 1) {
                Some(new_location) => {
                    if new_location.get_piece(board).piece_color == PieceColor::None {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                        if y == 1 {
                            match Location::check_bounds(x, y + 2) {
                                Some(new_location) => {
                                    if new_location.get_piece(board).piece_color == PieceColor::None
                                    {
                                        match validate_move(
                                            board,
                                            location,
                                            Location::from_x_y(x, y + 2),
                                            validate_check,
                                        ) {
                                            Some(new_location) => {
                                                valid_moves.push(new_location);
                                            }
                                            None => (),
                                        };
                                    }
                                }
                                None => (),
                            };
                        };
                    }
                }
                None => (),
            };

            let en_passant = match validate_check {
                true => unsafe { EN_PASSANT },
                false => None,
            };

            match Location::check_bounds(x + 1, y + 1) {
                Some(new_location) => {
                    if new_location.get_piece(board).piece_color == PieceColor::Black {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                    }

                    // en passant
                    match en_passant {
                        Some(en_passant_location) => {
                            if en_passant_location.to_x_y().0 == x + 1
                                && en_passant_location.to_x_y().1 == y + 1
                            {
                                match validate_move(
                                    board,
                                    location,
                                    Location::from_x_y(x + 1, y + 1),
                                    validate_check,
                                ) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            } else if en_passant_location.to_x_y().0 == x - 1
                                && en_passant_location.to_x_y().1 == y + 1
                            {
                                match validate_move(
                                    board,
                                    location,
                                    Location::from_x_y(x - 1, y + 1),
                                    validate_check,
                                ) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            }
                        }
                        None => (),
                    };
                }
                None => (),
            };

            match Location::check_bounds(x - 1, y + 1) {
                Some(new_location) => {
                    if new_location.get_piece(board).piece_color == PieceColor::Black {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                    }

                    // en passant
                    match en_passant {
                        Some(en_passant_location) => {
                            if en_passant_location.to_x_y().0 == x + 1
                                && en_passant_location.to_x_y().1 == y + 1
                            {
                                match validate_move(
                                    board,
                                    location,
                                    Location::from_x_y(x + 1, y + 1),
                                    validate_check,
                                ) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            } else if en_passant_location.to_x_y().0 == x - 1
                                && en_passant_location.to_x_y().1 == y + 1
                            {
                                match validate_move(
                                    board,
                                    location,
                                    Location::from_x_y(x - 1, y + 1),
                                    validate_check,
                                ) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            }
                        }
                        None => (),
                    };
                }
                None => (),
            };
        }

        PieceColor::Black => {
            // normal move
            match Location::check_bounds(x, y - 1) {
                Some(new_location) => {
                    if new_location.get_piece(board).piece_color == PieceColor::None {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                        if y == 6 {
                            // move 2 spaces
                            match Location::check_bounds(x, y - 2) {
                                Some(new_location) => {
                                    if new_location.get_piece(board).piece_color == PieceColor::None
                                    {
                                        match validate_move(
                                            board,
                                            location,
                                            Location::from_x_y(x, y - 2),
                                            validate_check,
                                        ) {
                                            Some(new_location) => {
                                                valid_moves.push(new_location);
                                            }
                                            None => (),
                                        };
                                    }
                                }
                                None => (),
                            };
                        };
                    }
                }
                None => (),
            };

            let en_passant = match validate_check {
                true => unsafe { EN_PASSANT },
                false => None,
            };

            // attack diagonally
            match Location::check_bounds(x + 1, y - 1) {
                Some(new_location) => {
                    if new_location.get_piece(board).piece_color == PieceColor::White {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                    }

                    // en passant
                    match en_passant {
                        Some(en_passant_location) => {
                            if en_passant_location.to_x_y().0 == x + 1
                                && en_passant_location.to_x_y().1 == y - 1
                            {
                                match validate_move(
                                    board,
                                    location,
                                    Location::from_x_y(x + 1, y - 1),
                                    validate_check,
                                ) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            } else if en_passant_location.to_x_y().0 == x - 1
                                && en_passant_location.to_x_y().1 == y - 1
                            {
                                match validate_move(
                                    board,
                                    location,
                                    Location::from_x_y(x - 1, y - 1),
                                    validate_check,
                                ) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            }
                        }
                        None => (),
                    };
                }
                None => (),
            };

            match Location::check_bounds(x - 1, y - 1) {
                Some(new_location) => {
                    if new_location.get_piece(board).piece_color == PieceColor::White {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                    }

                    // en passant
                    match en_passant {
                        Some(en_passant_location) => {
                            if en_passant_location.to_x_y().0 == x + 1
                                && en_passant_location.to_x_y().1 == y - 1
                            {
                                match validate_move(
                                    board,
                                    location,
                                    Location::from_x_y(x + 1, y - 1),
                                    validate_check,
                                ) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            } else if en_passant_location.to_x_y().0 == x - 1
                                && en_passant_location.to_x_y().1 == y - 1
                            {
                                match validate_move(
                                    board,
                                    location,
                                    Location::from_x_y(x - 1, y - 1),
                                    validate_check,
                                ) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            }
                        }
                        None => (),
                    };
                }
                None => (),
            };
        }

        PieceColor::None => (),
    }

    valid_moves
}

fn get_moves_knight(
    location: &Location,
    board: [Piece; 64],
    validate_check: bool,
) -> Vec<Location> {
    trace!("Getting moves for knight at {}", location.to_string());
    let mut valid_moves: Vec<Location> = Vec::new();

    let (x, y) = location.to_x_y();
    let x = x as i16;
    let y = y as i16;

    for i in -2..3 {
        for j in -2..3 {
            if (i as i16).abs() + (j as i16).abs() == 3 {
                match Location::check_bounds(x + i, y + j) {
                    Some(new_location) => {
                        if location.get_color(board) != new_location.get_color(board) {
                            match validate_move(board, location, new_location, validate_check) {
                                Some(new_location) => {
                                    valid_moves.push(new_location);
                                }
                                None => (),
                            };
                        }
                    }
                    None => (),
                };
            }
        }
    }

    valid_moves
}

fn get_moves_bishop(
    location: &Location,
    board: [Piece; 64],
    validate_check: bool,
) -> Vec<Location> {
    trace!("Getting moves for bishop at {}", location.to_string());
    let mut valid_moves: Vec<Location> = Vec::new();

    let (x, y) = location.to_x_y();
    let x = x as i16;
    let y = y as i16;

    let steps = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    for step in steps {
        let mut i = 1;
        loop {
            match Location::check_bounds(x + step.0 * i, y + step.1 * i) {
                Some(new_location) => {
                    if new_location.get_color(board) == PieceColor::None {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                    } else if new_location.get_color(board) != location.get_color(board) {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                        break;
                    } else {
                        break;
                    }
                }
                None => break,
            };
            i += 1;
        }
    }

    valid_moves
}

fn get_moves_rook(location: &Location, board: [Piece; 64], validate_check: bool) -> Vec<Location> {
    trace!("Getting moves for rook at {}", location.to_string());
    let mut valid_moves: Vec<Location> = Vec::new();

    let (x, y) = location.to_x_y();
    let x = x as i16;
    let y = y as i16;

    let steps = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for step in steps {
        let mut i = 1;
        loop {
            match Location::check_bounds(x + step.0 * i, y + step.1 * i) {
                Some(new_location) => {
                    if new_location.get_color(board) == PieceColor::None {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                    } else if new_location.get_color(board) != location.get_color(board) {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                        break;
                    } else {
                        break;
                    }
                }
                None => break,
            };
            i += 1;
        }
    }

    valid_moves
}

fn get_moves_queen(location: &Location, board: [Piece; 64], validate_check: bool) -> Vec<Location> {
    trace!("Getting moves for queen at {}", location.to_string());
    let mut valid_moves: Vec<Location> = Vec::new();

    let (x, y) = location.to_x_y();
    let x = x as i16;
    let y = y as i16;

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
        let mut i = 1;
        loop {
            match Location::check_bounds(x + step.0 * i, y + step.1 * i) {
                Some(new_location) => {
                    if new_location.get_color(board) == PieceColor::None {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                    } else if new_location.get_color(board) != location.get_color(board) {
                        match validate_move(board, location, new_location, validate_check) {
                            Some(new_location) => {
                                valid_moves.push(new_location);
                            }
                            None => (),
                        };
                        break;
                    } else {
                        break;
                    }
                }
                None => break,
            };
            i += 1;
        }
    }

    valid_moves
}

fn get_moves_king(location: &Location, board: [Piece; 64], validate_check: bool) -> Vec<Location> {
    trace!("Getting moves for king at {}", location.to_string());
    let mut valid_moves: Vec<Location> = Vec::new();

    let (x, y) = location.to_x_y();
    let x = x as i16;
    let y = y as i16;

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
        match Location::check_bounds(x + step.0, y + step.1) {
            Some(new_location) => {
                if new_location.get_color(board) != location.get_color(board) {
                    match validate_move(board, location, new_location, validate_check) {
                        Some(new_location) => {
                            valid_moves.push(new_location);
                        }
                        None => (),
                    };
                }
            }
            None => (),
        };
    }

    // castling
    match location.get_color(board) {
        PieceColor::White => {
            if location.to_string() == "e1" {
                let mut valid_castles = unsafe { &VALID_CASTLES };
                for castle in valid_castles {
                    if castle.0.get_color(board) == location.get_color(board) {
                        if castle.1.to_string() == "a1" {
                            if (Location::from_str("b1").get_color(board) == PieceColor::None
                                && Location::from_str("c1").get_color(board) == PieceColor::None
                                && Location::from_str("d1").get_color(board) == PieceColor::None)
                            {
                                let mut board_passing = board.clone();
                                board_passing[Location::from_str("d1").location] =
                                    board_passing[Location::from_str("e1").location];
                                board_passing[Location::from_str("e1").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                board_passing[Location::from_str("c1").location] =
                                    board_passing[Location::from_str("d1").location];
                                board_passing[Location::from_str("d1").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                board_passing[Location::from_str("b1").location] =
                                    board_passing[Location::from_str("c1").location];
                                board_passing[Location::from_str("c1").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                match validate_move(board, location, castle.1, validate_check) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            }
                        } else if castle.1.to_string() == "h1" {
                            if (Location::from_str("f1").get_color(board) == PieceColor::None
                                && Location::from_str("g1").get_color(board) == PieceColor::None)
                            {
                                let mut board_passing = board.clone();
                                board_passing[Location::from_str("f1").location] =
                                    board_passing[Location::from_str("e1").location];
                                board_passing[Location::from_str("e1").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                board_passing[Location::from_str("g1").location] =
                                    board_passing[Location::from_str("f1").location];
                                board_passing[Location::from_str("f1").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                match validate_move(board, location, castle.1, validate_check) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            }
                        }
                    }
                }
            }
        }
        PieceColor::Black => {
            if location.to_string() == "e8" {
                let mut valid_castles = unsafe { &VALID_CASTLES };

                for castle in valid_castles {
                    if castle.0.get_color(board) == location.get_color(board) {
                        if castle.1.to_string() == "a8" {
                            if (Location::from_str("b8").get_color(board) == PieceColor::None
                                && Location::from_str("c8").get_color(board) == PieceColor::None
                                && Location::from_str("d8").get_color(board) == PieceColor::None)
                            {
                                let mut board_passing = board.clone();
                                board_passing[Location::from_str("d8").location] =
                                    board_passing[Location::from_str("e8").location];
                                board_passing[Location::from_str("e8").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                board_passing[Location::from_str("c8").location] =
                                    board_passing[Location::from_str("d8").location];
                                board_passing[Location::from_str("d8").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                board_passing[Location::from_str("b8").location] =
                                    board_passing[Location::from_str("c8").location];
                                board_passing[Location::from_str("c8").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                match validate_move(board, location, castle.1, validate_check) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            }
                        } else if castle.1.to_string() == "h8" {
                            if (Location::from_str("f8").get_color(board) == PieceColor::None
                                && Location::from_str("g8").get_color(board) == PieceColor::None)
                            {
                                let mut board_passing = board.clone();
                                board_passing[Location::from_str("f8").location] =
                                    board_passing[Location::from_str("e8").location];
                                board_passing[Location::from_str("e8").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                board_passing[Location::from_str("g8").location] =
                                    board_passing[Location::from_str("f8").location];
                                board_passing[Location::from_str("f8").location] =
                                    Piece::new(PieceColor::None, PieceKind::None);
                                if get_in_check(board_passing, location.get_color(board)) {
                                    continue;
                                }

                                match validate_move(board, location, castle.1, validate_check) {
                                    Some(new_location) => {
                                        valid_moves.push(new_location);
                                    }
                                    None => (),
                                };
                            }
                        }
                    }
                }
            }
        }
        PieceColor::None => (),
    }
    valid_moves
}

fn validate_move(
    board: [Piece; 64],
    initial_location: &Location,
    next_location: Location,
    validate_check: bool,
) -> Option<Location> {
    let color = initial_location.get_color(board);
    let (x, y) = next_location.to_x_y();
    match Location::check_bounds(x, y) {
        Some(next_location) => {
            if validate_check {
                let mut new_board = board.clone();
                new_board[next_location.location] = new_board[initial_location.location];
                new_board[initial_location.location] =
                    Piece::new(PieceColor::None, PieceKind::None);

                if get_in_check(new_board, color) {
                    return None;
                }
            }

            return Some(next_location);
        }
        None => return Some(next_location),
    }
}

fn get_game_state() -> GameState {
    let mut white_moves: Vec<Location> = Vec::new();
    let mut black_moves: Vec<Location> = Vec::new();

    let board = (unsafe { &BOARD }).clone();

    for i in 0..64 {
        let location = Location { location: i };
        let piece = location.get_piece(board);
        if piece.piece_color == PieceColor::White {
            let moves = location.get_moves(board, true);
            for move_location in moves {
                white_moves.push(move_location);
            }
        } else if piece.piece_color == PieceColor::Black {
            let moves = location.get_moves(board, true);
            for move_location in moves {
                black_moves.push(move_location);
            }
        }
    }

    if white_moves.len() == 0 {
        if get_in_check(board, PieceColor::White) {
            return GameState::BlackCheckmate;
        } else {
            return GameState::Stalemate;
        }
    } else if black_moves.len() == 0 {
        if get_in_check(board, PieceColor::Black) {
            return GameState::WhiteCheckmate;
        } else {
            return GameState::Stalemate;
        }
    }

    GameState::Playing
}

fn get_in_check(board: [Piece; 64], color: PieceColor) -> bool {
    let mut king_location: Option<Location> = None;

    let mut location = 0;
    for piece in board {
        if piece.piece_color == color && piece.piece_kind == PieceKind::King {
            king_location = Some(Location { location: location });
            break;
        }
        location += 1;
    }

    let king_location = match king_location {
        Some(king_location) => king_location,
        None => return false,
    };

    let mut i = 0;
    for piece in board {
        if piece.piece_color == color {
        } else if piece.piece_color == PieceColor::None {
        } else {
            let moves = (Location { location: i }).get_moves(board, false);
            for try_move in moves {
                if try_move.to_x_y() == king_location.to_x_y() {
                    debug!(
                        "{} is dead by {} if {} moves to {}",
                        color.to_string(),
                        piece.piece_color.to_string(),
                        Location { location: i }.to_string(),
                        king_location.to_string()
                    );
                    return true;
                }
            }
        }
        i += 1;
    }

    return false;
}

fn flip_turn() {
    unsafe {
        match TURN {
            PieceColor::White => {
                TURN = PieceColor::Black;
            }
            PieceColor::Black => {
                TURN = PieceColor::White;
            }
            PieceColor::None => {
                TURN = PieceColor::White;
            }
        }
    }

    let state = get_game_state();

    match state {
        GameState::Playing => (),
        GameState::WhiteCheck => {
            info!("White is in check.");
        }
        GameState::BlackCheck => {
            info!("Black is in check.");
        }
        GameState::WhiteCheckmate => {
            info!("White checkmate.");
            // kill the program
            std::process::exit(0);
        }
        GameState::BlackCheckmate => {
            info!("Black checkmate.");
            // kill the program
            std::process::exit(0);
        }
        GameState::Stalemate => {
            info!("Stalemate.");
        }
        GameState::Error => {}
    }
}

fn remove_castle(location: Location) {
    match location.location {
        0 => (),
        7 => (),
        56 => (),
        63 => (),
        4 => (),
        60 => (),
        _ => return,
    }

    let valid_castles = (unsafe { &VALID_CASTLES }).clone();
    let mut new_valid_castles: Vec<(Location, Location)> = Vec::new();
    for castle in valid_castles {
        if castle.0.to_string() == location.to_string() {
        } else if castle.1.to_string() == location.to_string() {
        } else {
            new_valid_castles.push(castle);
        }
    }

    unsafe {
        VALID_CASTLES = new_valid_castles;
    }
}

#[tauri::command]
fn select_move(location: String) -> Vec<JsRequest> {
    let mut requests: Vec<JsRequest> = Vec::new();
    let selected_piece = unsafe { SELECTED_PIECE };
    let location = Location::from_string(location);
    let turn = unsafe { TURN };
    let board = (unsafe { BOARD }).clone();

    if location.get_color(board) == PieceColor::None {
        unsafe { SELECTED_PIECE = None };
        requests.push(JsRequest::new("removeHighlights".to_string()));

        match selected_piece {
            Some(selected_piece) => {
                if selected_piece.get_color(board) == turn {
                    let valid_moves = selected_piece.get_moves(board, true);
                    let mut is_valid_move = false;
                    for valid_move in valid_moves {
                        if valid_move.to_string() == location.to_string() {
                            is_valid_move = true;
                        }
                    }

                    if !is_valid_move {
                        return requests;
                    }

                    requests.push(
                        JsRequest::new("movePiece".to_string())
                            .add_argument(selected_piece.to_string())
                            .add_argument(location.to_string()),
                    );
                    let piece = selected_piece.get_piece(board);
                    set_piece(
                        board,
                        Piece::new(PieceColor::None, PieceKind::None),
                        selected_piece.to_x_y().0,
                        selected_piece.to_x_y().1,
                    );

                    set_piece(board, piece, location.to_x_y().0, location.to_x_y().1);
                    remove_castle(location);
                    remove_castle(location);

                    if piece.piece_kind == PieceKind::Pawn {
                        if location.to_x_y().1 == 0 || location.to_x_y().1 == 7 {
                            let mut piece = piece;
                            piece.piece_kind = PieceKind::Queen;
                            set_piece(board, piece, location.to_x_y().0, location.to_x_y().1);

                            requests.push(
                                JsRequest::new("removePiece".to_string())
                                    .add_argument(location.to_string()),
                            );
                            requests.push(create_piece(
                                "wq".to_string(),
                                location.to_x_y().0,
                                location.to_x_y().1,
                            ));
                        }

                        if selected_piece.to_x_y().1 - location.to_x_y().1 == 2 {
                            unsafe {
                                EN_PASSANT = Some(Location::from_x_y(
                                    location.to_x_y().0,
                                    location.to_x_y().1 + 1,
                                ))
                            };
                        } else if selected_piece.to_x_y().1 - location.to_x_y().1 == -2 {
                            unsafe {
                                EN_PASSANT = Some(Location::from_x_y(
                                    location.to_x_y().0,
                                    location.to_x_y().1 - 1,
                                ))
                            };
                        } else {
                            unsafe { EN_PASSANT = None };

                            let pass = match selected_piece.get_color(board) {
                                PieceColor::White => Some(Location::from_x_y(
                                    location.to_x_y().0,
                                    location.to_x_y().1 - 1,
                                )),
                                PieceColor::Black => Some(Location::from_x_y(
                                    location.to_x_y().0,
                                    location.to_x_y().1 + 1,
                                )),
                                PieceColor::None => None,
                            };

                            match pass {
                                Some(pass) => {
                                    if pass.get_piece(board).piece_kind == PieceKind::Pawn {
                                        requests.push(
                                            JsRequest::new("removePiece".to_string())
                                                .add_argument(pass.to_string()),
                                        );

                                        set_piece(
                                            board,
                                            Piece::new(PieceColor::None, PieceKind::None),
                                            pass.to_x_y().0,
                                            pass.to_x_y().1,
                                        );
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                    flip_turn();
                }
            }
            None => {}
        }
    } else if location.get_color(board) == turn {
        unsafe { SELECTED_PIECE = Some(location) };
        requests.push(JsRequest::new("removeHighlights".to_string()));

        let mut castle = false;
        match selected_piece {
            Some(selected_piece) => {
                let valid_moves = selected_piece.get_moves(board, true);
                if selected_piece.get_piece(board).piece_kind == PieceKind::King {
                    for valid_move in &valid_moves {
                        if location.to_string() == valid_move.to_string() {
                            castle = true;
                            break;
                        }
                    }
                }
            }
            None => {}
        }

        if castle {
            info!("Castle!");
            let rook = location;
            let king = selected_piece.unwrap();
            let (rook_x, rook_y) = rook.to_x_y();
            let (king_x, king_y) = king.to_x_y();
            let new_rook_x = match rook_x {
                0 => 3,
                7 => 5,
                _ => 0,
            };
            let new_king_x = match rook_x {
                0 => 2,
                7 => 6,
                _ => 0,
            };
            let new_rook = Location::from_x_y(new_rook_x, rook_y);
            let new_king = Location::from_x_y(new_king_x, king_y);
            let piece_king = king.get_piece(board);
            let piece_rook = rook.get_piece(board);
            remove_castle(king);

            set_piece(
                board,
                Piece::new(PieceColor::None, PieceKind::None),
                rook_x,
                rook_y,
            );
            set_piece(
                board,
                Piece::new(PieceColor::None, PieceKind::None),
                king_x,
                king_y,
            );
            set_piece(board, piece_king, new_king_x, king_y);
            set_piece(board, piece_rook, new_rook_x, rook_y);
            flip_turn();

            requests.push(
                JsRequest::new("movePiece".to_string())
                    .add_argument(rook.to_string())
                    .add_argument(new_rook.to_string()),
            );
            requests.push(
                JsRequest::new("movePiece".to_string())
                    .add_argument(king.to_string())
                    .add_argument(new_king.to_string()),
            );
            requests.push(JsRequest::new("removeHighlights".to_string()));
        } else {
            let valid_moves = location.get_moves(board, true);
            requests.push(create_highlight(
                location.to_x_y().0,
                location.to_x_y().1,
                0,
            ));
            for valid_move in valid_moves {
                requests.push(create_highlight(
                    valid_move.to_x_y().0,
                    valid_move.to_x_y().1,
                    1,
                ));
            }
        }
    } else if location.get_color(board) != turn {
        unsafe { SELECTED_PIECE = None };
        requests.push(JsRequest::new("removeHighlights".to_string()));

        match selected_piece {
            Some(selected_piece) => {
                if selected_piece.get_color(board) == turn {
                    let valid_moves = selected_piece.get_moves(board, true);
                    let mut is_valid_move = false;
                    for valid_move in valid_moves {
                        if valid_move.to_string() == location.to_string() {
                            is_valid_move = true;
                        }
                    }

                    if !is_valid_move {
                        return requests;
                    }

                    if location.get_color(board) == PieceColor::None {
                        requests.push(
                            JsRequest::new("movePiece".to_string())
                                .add_argument(selected_piece.to_string())
                                .add_argument(location.to_string()),
                        );
                        let piece = selected_piece.get_piece(board);
                        set_piece(
                            board,
                            Piece::new(PieceColor::None, PieceKind::None),
                            selected_piece.to_x_y().0,
                            selected_piece.to_x_y().1,
                        );

                        set_piece(board, piece, location.to_x_y().0, location.to_x_y().1);

                        // promotion
                        if piece.piece_kind == PieceKind::Pawn {
                            if location.to_x_y().1 == 0 || location.to_x_y().1 == 7 {
                                let mut piece = piece;
                                piece.piece_kind = PieceKind::Queen;
                                set_piece(board, piece, location.to_x_y().0, location.to_x_y().1);

                                requests.push(
                                    JsRequest::new("removePiece".to_string())
                                        .add_argument(location.to_string()),
                                );
                                requests.push(create_piece(
                                    "wq".to_string(),
                                    location.to_x_y().0,
                                    location.to_x_y().1,
                                ));
                            }
                        }

                        remove_castle(selected_piece);
                        flip_turn()
                    } else {
                        requests.push(
                            JsRequest::new("removePiece".to_string())
                                .add_argument(location.to_string()),
                        );
                        requests.push(
                            JsRequest::new("movePiece".to_string())
                                .add_argument(selected_piece.to_string())
                                .add_argument(location.to_string()),
                        );
                        let piece = selected_piece.get_piece(board);
                        set_piece(
                            board,
                            Piece::new(PieceColor::None, PieceKind::None),
                            selected_piece.to_x_y().0,
                            selected_piece.to_x_y().1,
                        );

                        set_piece(board, piece, location.to_x_y().0, location.to_x_y().1);

                        // promotion
                        if piece.piece_kind == PieceKind::Pawn {
                            if location.to_x_y().1 == 0 || location.to_x_y().1 == 7 {
                                let mut piece = piece;
                                piece.piece_kind = PieceKind::Queen;
                                set_piece(board, piece, location.to_x_y().0, location.to_x_y().1);

                                requests.push(
                                    JsRequest::new("removePiece".to_string())
                                        .add_argument(location.to_string()),
                                );
                                requests.push(create_piece(
                                    "wq".to_string(),
                                    location.to_x_y().0,
                                    location.to_x_y().1,
                                ));
                            }
                        }

                        remove_castle(selected_piece);
                        flip_turn();
                    }
                }
            }
            None => {}
        }
    }

    requests
}

fn new_board() {
    // add valid castling moves
    // first piece in tuple is king, second is rook
    let mut valid_castles: Vec<(Location, Location)> = Vec::new();
    valid_castles.push((Location::from_x_y(4, 0), Location::from_x_y(7, 0)));
    valid_castles.push((Location::from_x_y(4, 0), Location::from_x_y(0, 0)));
    valid_castles.push((Location::from_x_y(4, 7), Location::from_x_y(7, 7)));
    valid_castles.push((Location::from_x_y(4, 7), Location::from_x_y(0, 7)));
    unsafe { VALID_CASTLES = valid_castles };

    let mut board: [Piece; 64] = [Piece::new(PieceColor::None, PieceKind::None); 64];
    for i in 0..8 {
        set_piece(board, Piece::new(PieceColor::White, PieceKind::Pawn), i, 1);
        set_piece(board, Piece::new(PieceColor::Black, PieceKind::Pawn), i, 6);
    }

    set_piece(
        board,
        Piece::new(PieceColor::White, PieceKind::Knight),
        1,
        0,
    );
    set_piece(
        board,
        Piece::new(PieceColor::White, PieceKind::Knight),
        6,
        0,
    );
    set_piece(
        board,
        Piece::new(PieceColor::Black, PieceKind::Knight),
        1,
        7,
    );
    set_piece(
        board,
        Piece::new(PieceColor::Black, PieceKind::Knight),
        6,
        7,
    );
    set_piece(
        board,
        Piece::new(PieceColor::White, PieceKind::Bishop),
        2,
        0,
    );
    set_piece(
        board,
        Piece::new(PieceColor::White, PieceKind::Bishop),
        5,
        0,
    );
    set_piece(
        board,
        Piece::new(PieceColor::Black, PieceKind::Bishop),
        2,
        7,
    );
    set_piece(
        board,
        Piece::new(PieceColor::Black, PieceKind::Bishop),
        5,
        7,
    );
    set_piece(board, Piece::new(PieceColor::White, PieceKind::Rook), 0, 0);
    set_piece(board, Piece::new(PieceColor::White, PieceKind::Rook), 7, 0);
    set_piece(board, Piece::new(PieceColor::Black, PieceKind::Rook), 0, 7);
    set_piece(board, Piece::new(PieceColor::Black, PieceKind::Rook), 7, 7);
    set_piece(board, Piece::new(PieceColor::White, PieceKind::Queen), 3, 0);
    set_piece(board, Piece::new(PieceColor::Black, PieceKind::Queen), 3, 7);
    set_piece(board, Piece::new(PieceColor::White, PieceKind::King), 4, 0);
    set_piece(board, Piece::new(PieceColor::Black, PieceKind::King), 4, 7);
}

fn create_highlight(x: i16, y: i16, kind: i16) -> JsRequest {
    JsRequest::new("createHighlight".to_string())
        .add_argument((x * 64).to_string() + "px")
        .add_argument((y * 64).to_string() + "px")
        .add_argument(kind.to_string())
}

fn create_piece(piece: String, x: i16, y: i16) -> JsRequest {
    let id = to_algebraic((x, y));
    let svg_path = "src-tauri/assets/pieces/".to_string() + &piece + ".svg";
    let x_in_px = (x * 64).to_string() + "px";
    let y_in_px = (y * 64).to_string() + "px";
    JsRequest::new("createPiece".to_string())
        .add_argument(id)
        .add_argument(svg_path)
        .add_argument(x_in_px)
        .add_argument(y_in_px)
}

#[tauri::command]
fn setup_board(_window: Window) -> Vec<JsRequest> {
    new_board();
    let mut requests: Vec<JsRequest> = Vec::new();
    for i in 0..8 {
        requests.push(create_piece("wp".to_string(), i, 1));
        requests.push(create_piece("bp".to_string(), i, 6));
    }
    requests.push(create_piece("wn".to_string(), 1, 0));
    requests.push(create_piece("wn".to_string(), 6, 0));
    requests.push(create_piece("bn".to_string(), 1, 7));
    requests.push(create_piece("bn".to_string(), 6, 7));
    requests.push(create_piece("wb".to_string(), 2, 0));
    requests.push(create_piece("wb".to_string(), 5, 0));
    requests.push(create_piece("bb".to_string(), 2, 7));
    requests.push(create_piece("bb".to_string(), 5, 7));
    requests.push(create_piece("wr".to_string(), 0, 0));
    requests.push(create_piece("wr".to_string(), 7, 0));
    requests.push(create_piece("br".to_string(), 0, 7));
    requests.push(create_piece("br".to_string(), 7, 7));
    requests.push(create_piece("wq".to_string(), 3, 0));
    requests.push(create_piece("bq".to_string(), 3, 7));
    requests.push(create_piece("wk".to_string(), 4, 0));
    requests.push(create_piece("bk".to_string(), 4, 7));

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
            setup_board,
            select_move,
            utils::js_error,
            utils::js_warn,
            utils::js_info,
            utils::js_debug,
            utils::js_trace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
