// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{env, fmt::Display};

use log::LevelFilter;
use serde::{Deserialize, Serialize};
use tauri::{PhysicalSize, Window};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use ts_rs::TS;

use crate::ai::{ChessAI, MinimaxAI, RandomAI};

mod ai;
mod utils;

enum GameState {
    Playing,
    WhiteInCheck,
    BlackInCheck,
    WhiteInCheckmate,
    BlackInCheckmate,
    Stalemate,
    Other,
}

// PieceColor enum
// used to determine what color the piece is

#[derive(TS, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[ts(export, export_to = "../src/bindings/PieceColor.ts")]
enum PieceColor {
    White,
    Black,
    None,
}

impl PieceColor {
    fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
            PieceColor::None => PieceColor::None,
        }
    }

    fn random() -> PieceColor {
        match rand::random::<bool>() {
            true => PieceColor::White,
            false => PieceColor::Black,
        }
    }
}

impl Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceColor::White => write!(f, "White"),
            PieceColor::Black => write!(f, "Black"),
            PieceColor::None => write!(f, "None"),
        }
    }
}

// PieceKind enum
// used to determine what kind of piece it is

#[derive(TS, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[ts(export, export_to = "../src/bindings/PieceKind.ts")]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None,
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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

// Piece struct
// mainly used for color and kind

#[derive(TS, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[ts(export, export_to = "../src/bindings/Piece.ts")]
struct Piece {
    position: Position,
    color: PieceColor,
    kind: PieceKind,
    pid: i8,
}

impl Piece {
    fn none() -> Piece {
        Piece {
            position: Position::none(),
            color: PieceColor::None,
            kind: PieceKind::None,
            pid: -1,
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.color, self.kind)
    }
}

#[derive(TS, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[ts(export, export_to = "../src/bindings/Position.ts")]
struct Position {
    value: i8,
}

impl Position {
    fn new(value: i8) -> Position {
        Position { value: value }
    }

    fn from_x_y(x: i8, y: i8) -> Position {
        Position { value: x + (y * 8) }
    }

    fn x_y(&self) -> (i8, i8) {
        (self.x(), self.y())
    }

    fn x(&self) -> i8 {
        self.value % 8
    }

    fn y(&self) -> i8 {
        self.value / 8
    }

    fn from_algebraic(algebraic: &String) -> Position {
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
            _ => 0,
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
            _ => 0,
        };
        Position::from_x_y(x, y)
    }

    fn algebraic(&self) -> String {
        let mut algebraic = String::new();
        algebraic.push(match self.x() {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => 'a',
        });
        algebraic.push(match self.y() {
            0 => '1',
            1 => '2',
            2 => '3',
            3 => '4',
            4 => '5',
            5 => '6',
            6 => '7',
            7 => '8',
            _ => '1',
        });
        algebraic
    }

    fn none() -> Position {
        Position { value: -1 }
    }

    fn mov(&self, delta: (i8, i8)) -> Option<Position> {
        let x = self.x() + delta.0;
        let y = self.y() + delta.1;

        if check_bounds((x, y)) {
            return Some(Position::from_x_y(x, y));
        }

        None
    }
}

fn check_bounds(position: (i8, i8)) -> bool {
    if position.0 < 0 || position.0 > 7 || position.1 < 0 || position.1 > 7 {
        return false;
    }
    true
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.algebraic())
    }
}

// Board struct
// use ts-rs
#[derive(TS, Serialize, Deserialize, Clone, PartialEq)]
#[ts(export, export_to = "../src/bindings/Board.ts")]
struct Board {
    pieces: Vec<Piece>,
    selected_piece: Option<Piece>,
    en_passant: Option<Piece>,
    white_castle_kingside: bool,
    white_castle_queenside: bool,
    black_castle_kingside: bool,
    black_castle_queenside: bool,
    turn: PieceColor,
    ai: PieceColor,
    max_pid: i8,
}

impl Board {
    // empty_board function
    // returns an empty board
    fn empty_board() -> Board {
        let mut pieces = Vec::new();
        for i in 0..64 {
            pieces.push(Piece {
                position: Position { value: i },
                color: PieceColor::None,
                kind: PieceKind::None,
                pid: -1,
            });
        }
        Board {
            pieces: pieces,
            selected_piece: None,
            en_passant: None,
            white_castle_kingside: false,
            white_castle_queenside: false,
            black_castle_kingside: false,
            black_castle_queenside: false,
            turn: PieceColor::None,
            ai: PieceColor::None,
            max_pid: 0,
        }
    }

    // default_board function
    // creates the typical starting chess board
    fn default_board() -> Board {
        let mut pid = 0;
        let mut pieces = Vec::new();
        for i in 0..64 {
            let mut color = PieceColor::None;
            let mut kind = PieceKind::None;
            if i >= 8 && i < 16 {
                color = PieceColor::White;
                kind = PieceKind::Pawn;
            } else if i >= 48 && i < 56 {
                color = PieceColor::Black;
                kind = PieceKind::Pawn;
            } else if i == 0 || i == 7 {
                color = PieceColor::White;
                kind = PieceKind::Rook;
            } else if i == 1 || i == 6 {
                color = PieceColor::White;
                kind = PieceKind::Knight;
            } else if i == 2 || i == 5 {
                color = PieceColor::White;
                kind = PieceKind::Bishop;
            } else if i == 3 {
                color = PieceColor::White;
                kind = PieceKind::Queen;
            } else if i == 4 {
                color = PieceColor::White;
                kind = PieceKind::King;
            } else if i == 56 || i == 63 {
                color = PieceColor::Black;
                kind = PieceKind::Rook;
            } else if i == 57 || i == 62 {
                color = PieceColor::Black;
                kind = PieceKind::Knight;
            } else if i == 58 || i == 61 {
                color = PieceColor::Black;
                kind = PieceKind::Bishop;
            } else if i == 59 {
                color = PieceColor::Black;
                kind = PieceKind::Queen;
            } else if i == 60 {
                color = PieceColor::Black;
                kind = PieceKind::King;
            }

            let mut new_pid = -1;
            if color != PieceColor::None && kind != PieceKind::None {
                new_pid = pid;
                pid += 1;
            }

            pieces.push(Piece {
                position: Position { value: i },
                color: color,
                kind: kind,
                pid: new_pid,
            });
        }
        Board {
            pieces: pieces,
            selected_piece: None,
            en_passant: None,
            white_castle_kingside: true,
            white_castle_queenside: true,
            black_castle_kingside: true,
            black_castle_queenside: true,
            turn: PieceColor::White,
            ai: PieceColor::Black,
            max_pid: 0,
        }
    }

    fn remove(&mut self, position: Position) -> Piece {
        let piece = self.get(&position);
        let mut removed = Piece::none();
        removed.position = position;
        self.set(removed);
        piece
    }

    fn set(&mut self, piece: Piece) -> Piece {
        let old_piece = self.get(&piece.position);
        self.pieces[piece.position.value as usize] = piece;
        old_piece
    }

    fn get(&self, position: &Position) -> Piece {
        self.pieces[position.value as usize]
    }

    fn mov(&mut self, from: Position, to: Position) -> Piece {
        let piece = self.get(&from);
        self.set(Piece {
            position: from,
            color: PieceColor::None,
            kind: PieceKind::None,
            pid: -1,
        });
        self.set(Piece {
            position: to,
            color: piece.color,
            kind: piece.kind,
            pid: piece.pid,
        })
    }

    fn create(&mut self, color: PieceColor, kind: PieceKind) -> Piece {
        let piece = Piece {
            position: Position { value: 0 },
            color: color,
            kind: kind,
            pid: self.max_pid,
        };
        self.max_pid += 1;
        piece
    }

    // Check castle does NOT verify if a castle is safe or not
    // The function only checks to see if the two pieces selected are valid castling pieces
    fn check_castle(&self, king: Piece, rook: Piece) -> bool {
        let color = king.color;

        if (rook.position.x() != 0 && rook.position.x() != 7)
            || (rook.position.y() != 0 && rook.position.y() != 7)
        {
            return false;
        }

        if (color == PieceColor::White) {
            if (self.white_castle_kingside || self.white_castle_queenside) {
                if (king.position.y() == rook.position.y()) {
                    if (king.position.x() < rook.position.x()) {
                        if (self.white_castle_kingside) {
                            return true;
                        }
                    } else {
                        if (self.white_castle_queenside) {
                            return true;
                        }
                    }
                }
            } else {
            }
        } else {
            if (self.black_castle_kingside || self.black_castle_queenside) {
                if (king.position.y() == rook.position.y()) {
                    if (king.position.x() < rook.position.x()) {
                        if (self.black_castle_kingside) {
                            return true;
                        }
                    } else {
                        if (self.black_castle_queenside) {
                            return true;
                        }
                    }
                }
            } else {
            }
        }
        false
    }

    fn get_moves(&self, position: &Position, check_check: bool) -> Vec<Position> {
        match self.get(position).kind {
            PieceKind::Pawn => get_moves_pawn(self, position, check_check),
            PieceKind::Knight => get_moves_knight(self, position, check_check),
            PieceKind::Bishop => get_moves_bishop(self, position, check_check),
            PieceKind::Rook => get_moves_rook(self, position, check_check),
            PieceKind::Queen => get_moves_queen(self, position, check_check),
            PieceKind::King => get_moves_king(self, position, check_check),
            PieceKind::None => Vec::new(),
        }
    }

    fn check_in_check(&self, color: PieceColor) -> bool {
        let mut king_position = Position::none();
        for piece in self.pieces.iter() {
            if piece.color == color && piece.kind == PieceKind::King {
                king_position = piece.position;
                break;
            }
        }
        let mut in_check = false;
        for piece in self.pieces.iter() {
            if piece.color != color {
                let moves = self.get_moves(&piece.position, false);
                for position in moves.iter() {
                    if *position == king_position {
                        in_check = true;
                        break;
                    }
                }
            }
        }
        in_check
    }

    fn validate(&self, from: Position, to: Position, check_check: bool) -> bool {
        if !check_bounds((from.x(), from.y())) || !check_bounds((to.x(), to.y())) {
            return false;
        }

        if check_check {
            let mut new_board = self.clone();
            new_board.mov(from, to);
            if new_board.check_in_check(self.turn) {
                return false;
            }
        }

        true
    }

    fn get_game_state(&self) -> GameState {
        let mut white_in_check = false;
        let mut black_in_check = false;

        let mut white_in_checkmate = true;
        let mut black_in_checkmate = true;

        let mut stalemate = true;

        for piece in self.pieces.iter() {
            if piece.color == PieceColor::White {
                if piece.kind == PieceKind::King {
                    if self.check_in_check(PieceColor::White) {
                        white_in_check = true;
                        stalemate = false;
                    }
                } else {
                    let moves = self.get_moves(&piece.position, true);
                    if moves.len() > 0 {
                        white_in_checkmate = false;
                        stalemate = false;
                    }
                }
            } else if piece.color == PieceColor::Black {
                if piece.kind == PieceKind::King {
                    if self.check_in_check(PieceColor::Black) {
                        black_in_check = true;
                        stalemate = false;
                    }
                } else {
                    let moves = self.get_moves(&piece.position, true);
                    if moves.len() > 0 {
                        black_in_checkmate = false;
                        stalemate = false;
                    }
                }
            }
        }

        if white_in_checkmate {
            return GameState::WhiteInCheckmate;
        } else if black_in_checkmate {
            return GameState::BlackInCheckmate;
        } else if white_in_check {
            return GameState::WhiteInCheck;
        } else if black_in_check {
            return GameState::BlackInCheck;
        } else if stalemate {
            return GameState::Stalemate;
        }

        GameState::Playing
    }

    fn execute_move(&self, to: &Position, from: &Position) -> Option<Board> {
        let piece = self.get(from);
        if piece.color != self.turn {
            // invalid move, wrong turn
            return None;
        }

        let mut new_board = self.clone();
        let moves = self.get_moves(from, true);
        let mut valid = false;

        for position in moves.iter() {
            if *position == *to {
                valid = true;
                break;
            }
        }

        if !valid {
            // invalid move, not a valid move
            return None;
        }

        // ok the move is valid, now we just need to make sure that when the piece is moved, we are making sure to update en passant, castling, etc.
        if piece.kind == PieceKind::Pawn {
            if piece.position.y() == 1 || piece.position.y() == 6 {
                //check en passant

                if piece.color == PieceColor::White {
                    if piece.position.y() == 1 && to.y() == 3 {
                        new_board.en_passant = Some(Piece {
                            position: Position::from_x_y(to.x(), 2),
                            color: PieceColor::None,
                            kind: PieceKind::None,
                            pid: -1,
                        });
                    }
                } else {
                    if piece.position.y() == 6 && to.y() == 4 {
                        new_board.en_passant = Some(Piece {
                            position: Position::from_x_y(to.x(), 5),
                            color: PieceColor::None,
                            kind: PieceKind::None,
                            pid: -1,
                        });
                    }
                }
            } else {
                // check if piece killed is en passant
                let dir = match piece.color {
                    PieceColor::White => 1,
                    PieceColor::Black => -1,
                    PieceColor::None => 0,
                };
                match new_board.en_passant {
                    Some(en_passant) => {
                        if en_passant.position == *to {
                            // en passant!
                            let mut killed = en_passant;
                            killed.position = Position::from_x_y(to.x(), from.y() - dir);
                            new_board.remove(killed.position);
                        }
                    }
                    None => {}
                }
            }

            new_board.en_passant = None;
            new_board.mov(*from, *to);
        } else {
            new_board.en_passant = None;
            if piece.kind == PieceKind::King {
                // check if move is a castling move or if it is just a normal move
                if self.check_castle(new_board.get(from), new_board.get(to)) {
                    // YES! it is a castling move

                    // now we need to figure out which rook is being moved

                    let side = match to.x() {
                        0 => "queenside",
                        7 => "kingside",
                        _ => "none",
                    };

                    if piece.color == PieceColor::White {
                        new_board.white_castle_kingside = false;
                        new_board.white_castle_queenside = false;
                    } else {
                        new_board.black_castle_kingside = false;
                        new_board.black_castle_queenside = false;
                    }

                    // now we need to move the pieces
                    let rook_new_position = match side {
                        "queenside" => Position::from_x_y(3, to.y()),
                        "kingside" => Position::from_x_y(5, to.y()),
                        _ => Position::none(),
                    };

                    let king_new_position = match side {
                        "queenside" => Position::from_x_y(2, to.y()),
                        "kingside" => Position::from_x_y(6, to.y()),
                        _ => Position::none(),
                    };

                    new_board.mov(*to, rook_new_position);
                    new_board.mov(*from, king_new_position);
                } else {
                    new_board.mov(*from, *to);
                }
            } else if piece.kind == PieceKind::Rook {
                // mostly normal, just need to make sure that castling is disabled
                let side = match piece.position.x() {
                    0 => "queenside",
                    7 => "kingside",
                    _ => "none",
                };

                if side == "queenside" {
                    if piece.color == PieceColor::White {
                        new_board.white_castle_queenside = false;
                    } else {
                        new_board.black_castle_queenside = false;
                    }
                } else if side == "kingside" {
                    if piece.color == PieceColor::White {
                        new_board.white_castle_kingside = false;
                    } else {
                        new_board.black_castle_kingside = false;
                    }
                }

                new_board.mov(*from, *to);
            } else {
                new_board.mov(*from, *to);
            }
        }

        new_board.turn = new_board.turn.opposite();
        new_board.selected_piece = None;
        Some(new_board)
    }
}

fn get_moves_pawn(board: &Board, position: &Position, check_check: bool) -> Vec<Position> {
    let mut moves = Vec::new();
    let piece = board.get(position);
    let direction = match piece.color {
        PieceColor::White => 1,
        PieceColor::Black => -1,
        PieceColor::None => {
            return moves;
        }
    };

    // single move forward
    let mut mov = position.mov((0, 1 * direction));
    match mov {
        Some(new_position) => {
            if board.get(&new_position).color == PieceColor::None {
                if (board.validate(*position, new_position, check_check)) {
                    moves.push(new_position);

                    mov = new_position.mov((0, 1 * direction));

                    // double move forward
                    if (piece.color == PieceColor::White && position.y() == 1)
                        || (piece.color == PieceColor::Black && position.y() == 6)
                    {
                        mov = position.mov((0, 2 * direction));
                        match mov {
                            Some(new_position) => {
                                if board.get(&new_position).color == PieceColor::None {
                                    if (board.validate(*position, new_position, check_check)) {
                                        moves.push(new_position);
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                }
            }
        }
        None => {}
    }

    // attack left
    mov = position.mov((-1, 1 * direction));
    match mov {
        Some(new_position) => {
            if board.get(&new_position).color != piece.color
                && board.get(&new_position).color != PieceColor::None
            {
                if (board.validate(*position, new_position, check_check)) {
                    moves.push(new_position);
                }
            }

            // en passant
            match board.en_passant {
                Some(en_passant) => {
                    if en_passant.position == Position::from_x_y(new_position.x(), new_position.y())
                    {
                        if (board.validate(*position, new_position, check_check)) {
                            moves.push(new_position);
                        }
                    }
                }
                None => {}
            }
        }
        None => {}
    }

    // attack right
    mov = position.mov((1, 1 * direction));
    match mov {
        Some(new_position) => {
            if board.get(&new_position).color != piece.color
                && board.get(&new_position).color != PieceColor::None
            {
                if (board.validate(*position, new_position, check_check)) {
                    moves.push(new_position);
                }
            }

            // en passant
            match board.en_passant {
                Some(en_passant) => {
                    if en_passant.position == Position::from_x_y(new_position.x(), new_position.y())
                    {
                        if (board.validate(*position, new_position, check_check)) {
                            moves.push(new_position);
                        }
                    }
                }
                None => {}
            }
        }
        None => {}
    }

    moves
}

fn get_moves_knight(board: &Board, position: &Position, check_check: bool) -> Vec<Position> {
    let mut moves = Vec::new();

    let deltas = vec![
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    for delta in deltas.iter() {
        let mov = position.mov(*delta);
        match mov {
            Some(new_position) => {
                if board.get(&new_position).color != board.get(position).color {
                    if (board.validate(*position, new_position, check_check)) {
                        moves.push(new_position);
                    }
                }
            }
            None => {}
        }
    }

    moves
}

fn get_moves_scanline(
    board: &Board,
    position: &Position,
    deltas: Vec<(i8, i8)>,
    check_check: bool,
) -> Vec<Position> {
    let mut moves = Vec::new();

    for delta in deltas.iter() {
        for distance in 1..8 {
            let mov = position.mov((delta.0 * distance, delta.1 * distance));
            match mov {
                Some(new_position) => {
                    if board.get(&new_position).color == PieceColor::None {
                        if (board.validate(*position, new_position, check_check)) {
                            moves.push(new_position);
                        }
                    } else if board.get(&new_position).color == board.get(position).color.opposite()
                    {
                        if (board.validate(*position, new_position, check_check)) {
                            moves.push(new_position);
                        }
                        break;
                    } else {
                        break;
                    }
                }
                None => {}
            }
        }
    }

    moves
}

fn get_moves_bishop(board: &Board, position: &Position, check_check: bool) -> Vec<Position> {
    let deltas = vec![(1, 1), (1, -1), (-1, -1), (-1, 1)];

    get_moves_scanline(board, position, deltas, check_check)
}

fn get_moves_rook(board: &Board, position: &Position, check_check: bool) -> Vec<Position> {
    let deltas = vec![(1, 0), (0, 1), (-1, 0), (0, -1)];

    get_moves_scanline(board, position, deltas, check_check)
}

fn get_moves_queen(board: &Board, position: &Position, check_check: bool) -> Vec<Position> {
    let deltas = vec![
        (1, 1),
        (1, -1),
        (-1, -1),
        (-1, 1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
    ];

    get_moves_scanline(board, position, deltas, check_check)
}

fn get_moves_king(board: &Board, position: &Position, check_check: bool) -> Vec<Position> {
    let mut moves = Vec::new();

    let deltas = vec![
        (1, 1),
        (1, -1),
        (-1, -1),
        (-1, 1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
    ];

    for delta in deltas.iter() {
        let mov = position.mov(*delta);
        match mov {
            Some(new_position) => {
                if board.get(&new_position).color != board.get(position).color {
                    if (board.validate(*position, new_position, check_check)) {
                        moves.push(new_position);
                    }
                }
            }
            None => {}
        }
    }

    let mut castle_queenside = false;
    let mut castle_kingside = false;

    // check for castles
    if board.get(position).color == PieceColor::White {
        castle_queenside = board.white_castle_queenside;
        castle_kingside = board.white_castle_kingside;
    } else {
        castle_queenside = board.black_castle_queenside;
        castle_kingside = board.black_castle_kingside;
    }

    if castle_queenside {
        let mut valid = true;
        for x_delta in 1..4 {
            let mut new_position;
            match position.mov((-x_delta, 0)) {
                Some(position) => new_position = position,
                None => break,
            }

            if board.get(&new_position).color != PieceColor::None {
                valid = false;
                break;
            }

            // validate check
            if (!board.validate(*position, new_position, check_check)) {
                valid = false;
                break;
            }
        }

        if valid {
            moves.push(Position::from_x_y(0, position.y()));
        }
    }

    if castle_kingside {
        let mut valid = true;
        for x_delta in 1..3 {
            let mut new_position;
            match position.mov((x_delta, 0)) {
                Some(position) => new_position = position,
                None => break,
            }

            if board.get(&new_position).color != PieceColor::None {
                valid = false;
                break;
            }

            // validate check
            if (!board.validate(*position, new_position, check_check)) {
                valid = false;
                break;
            }
        }

        if valid {
            moves.push(Position::from_x_y(7, position.y()));
        }
    }

    moves
}

// Tauri command: setup_board
// returns the default chess board
#[tauri::command]
fn setup_board(window: Window) -> Board {
    trace!("-> setup_board");

    Board::default_board()
}

fn select_piece(window: &Window, board: &mut Board, square: &String) {
    board.selected_piece = Some(board.get(&(Position::from_algebraic(&square))));
}

fn update_react_board(window: &Window, board: &Board) {
	trace!("<- update-board");
    let _update_board = window.emit("update-board", board);
}

fn update_react_selection(window: &Window, board: &Board, square: &String) {
	trace!("<- select-square");
    let _update_selection = match board.selected_piece {
        Some(piece) => window.emit("select-square", piece.position.algebraic()),
        None => window.emit("select-square", String::from("none")),
    };

    if board.selected_piece.is_some() {
        let piece = board.get(&(Position::from_algebraic(&square)));
        let moves = board.get_moves(&piece.position, true);
        let moves_algebraic = moves
            .iter()
            .map(|position| position.algebraic())
            .collect::<Vec<String>>();
		trace!("<- set-highlights");
        let _update_highlights = window.emit("set-highlights", moves_algebraic);
    } else {
		trace!("<- set-highlights");
        let _update_highlights = window.emit("set-highlights", Vec::<String>::new());
    }
}

// Tauri command: pick_square
// picks a square on the board
//
// params:
// window: the window where events will be emitted
// board: the board to pick a square on
// square: the square to pick
#[tauri::command]
async fn pick_square(window: Window, mut board: Board, square: String) {
    trace!("-> pick_square");

    if board.turn == board.ai {
        return;
    }

    let position = Position::from_algebraic(&square);
    let piece = board.get(&position);
    let color = piece.color;
    let turn = board.turn;
    match board.selected_piece {
        Some(selected_piece) => {
            if color == turn {
                // need to check for castles here. If this is not a castle then we can select the piece
                if selected_piece.kind == PieceKind::King
                    && board.check_castle(selected_piece, piece)
                {
					let new_board = board.execute_move(&position, &selected_piece.position);
					match new_board {
						Some(new_board) => {
							board = new_board;
						}
						None => {
							board.selected_piece = None;
						}
					}
                    
					update_react_selection(&window, &board, &square);
                } else {
                    // not a castle, so we can select the piece
                    board.selected_piece = Some(board.get(&(Position::from_algebraic(&square))));
                    update_react_selection(&window, &board, &square);
                }
            } else {
				match board.execute_move(&position, &selected_piece.position) {
					Some(new_board) => {
						board = new_board;
					}
					None => {
						board.selected_piece = None;
					}
				}

				update_react_selection(&window, &board, &square);
            }
        }
        None => {
            // no piece currently selected. if the new piece is a piece of the current turn, then we can select it
            if color == turn {
                board.selected_piece = Some(board.get(&(Position::from_algebraic(&square))));
                update_react_selection(&window, &board, &square);
            }
        }
    }

    update_react_board(&window, &board);

    if board.turn == board.ai {
        let ai: MinimaxAI = MinimaxAI::new(3);

        board = ai.get_move(&window, &board).await;
        update_react_board(&window, &board);
    }
}

// main function
// used to setup Tauri, nothing else
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
            pick_square,
            setup_board,
            utils::js_error,
            utils::js_warn,
            utils::js_info,
            utils::js_debug,
            utils::js_trace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
