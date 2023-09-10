use log::debug;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Board, Bot, Engine, GameState, MoveKind, Piece, PieceColor, PieceKind, Position};

#[derive(TS, Clone, Serialize, Deserialize)]
enum CastleSide {
    King,
    Queen,
    None,
}

#[derive(TS, Clone, Serialize, Deserialize)]
#[ts(export, export_to = "bindings/Alpha.ts")]
pub struct EngineAlpha {
    name: String,
}

impl Engine for EngineAlpha {
    fn new() -> Self {
        EngineAlpha {
            name: String::from("Alpha"),
        }
    }

    fn get_moves(&self, board: &Board, position: Position) -> Vec<Position> {
        get_moves_internal(board, position, false)
    }

    fn validate_move(&self, board: &Board, from: Position, to: Position) -> bool {
        let moves = get_moves_internal(board, from, false);
        for mov in moves {
            if mov.compare(&to) {
                return true;
            }
        }
        false
    }

    fn execute_move(&self, board: &mut Board, from: Position, to: Position) -> MoveKind {
        let from_piece = board.get_piece(&from);
        let to_piece = board.get_piece(&to);

        let mut move_kind = match to_piece.kind {
            PieceKind::None => MoveKind::Move,
            _ => MoveKind::Capture,
        };

        if from_piece.kind == PieceKind::Pawn {
            if to.y == 0 || to.y == 7 {
                // Pawn promotion
                move_kind = MoveKind::Promotion;

                board.set_piece(
                    &from,
                    Piece {
                        kind: PieceKind::None,
                        color: PieceColor::None,
                    },
                );
                board.set_piece(
                    &to,
                    Piece {
                        kind: PieceKind::Queen,
                        color: from_piece.color,
                    },
                );
            } else if from.x != to.x
                && to_piece.kind == PieceKind::None
                && board.en_passant.is_some()
            {
                // En passant
                move_kind = MoveKind::EnPassant;

                let color = match board.get_piece(&from).color {
                    PieceColor::White => PieceColor::Black,
                    PieceColor::Black => PieceColor::White,
                    _ => PieceColor::None,
                };

                let en_passant = board.en_passant.unwrap();
                board.move_piece(&from, &to);

                let direction = match color {
                    PieceColor::White => 1,
                    PieceColor::Black => -1,
                    _ => 0,
                };

                let passed = Position::from_x_y(to.x, to.y + direction);
                board.set_piece(
                    &passed,
                    Piece {
                        kind: PieceKind::None,
                        color: PieceColor::None,
                    },
                );
            } else if (to.y - from.y).abs() > 1 {
                // "Dash" move - two squares forward - sets en passant
                move_kind = MoveKind::Dash;

                board.move_piece(&from, &to);
                let passed = Position::from_x_y(to.x, (to.y + from.y) / 2);
                board.en_passant = Some(passed);
            } else {
                // Normal pawn move
                board.move_piece(&from, &to);
            }
        } else if from_piece.kind == PieceKind::King {
            if (from.x - to.x).abs() > 1 {
                // Castle
                move_kind = MoveKind::Castle;

                let king_from = from;
                let rook_from = to;

                let king_to = match from.x - to.x {
                    2 => king_from.add_delta((-2, 0)),
                    _ => king_from.add_delta((2, 0)),
                };

                let rook_to = match from.x - to.x {
                    2 => rook_from.add_delta((3, 0)),
                    _ => rook_from.add_delta((-2, 0)),
                };

                board.move_piece(&king_from, &king_to);
                board.move_piece(&rook_from, &rook_to);
            } else {
                // Normal king move
                board.move_piece(&from, &to);
            }

            // Remove castling rights for any king move
            match from_piece.color {
                PieceColor::White => {
                    board.castles.0 = false;
                    board.castles.1 = false;
                }
                PieceColor::Black => {
                    board.castles.2 = false;
                    board.castles.3 = false;
                }
                _ => (),
            }
        } else if from_piece.kind == PieceKind::Rook {
            // Rook move, need to remove castling rights
            match from {
                Position { x: 0, y: 0 } => board.castles.1 = false,
                Position { x: 7, y: 0 } => board.castles.0 = false,
                Position { x: 0, y: 7 } => board.castles.3 = false,
                Position { x: 7, y: 7 } => board.castles.2 = false,
                _ => (),
            }

            board.move_piece(&from, &to);
        } else {
            // Normal move
            board.move_piece(&from, &to);
        }

        board.turn = board.turn.opposite();
        if move_kind != MoveKind::Dash {
            board.en_passant = None;
        }

        move_kind
    }

    fn get_move_kind(&self, board: &Board, from: Position, to: Position) -> MoveKind {
        let from_piece = board.get_piece(&from);
        let to_piece = board.get_piece(&to);

        let mut move_kind = match to_piece.kind {
            PieceKind::None => MoveKind::Move,
            _ => MoveKind::Capture,
        };

        if from_piece.kind == PieceKind::Pawn {
            if to.y == 0 || to.y == 7 {
                move_kind = MoveKind::Promotion;
            } else if from.x != to.x && to_piece.kind == PieceKind::None {
                move_kind = MoveKind::EnPassant;
            }
        } else if from_piece.kind == PieceKind::King {
            if (from.x - to.x).abs() > 1 {
                move_kind = MoveKind::Castle;
            }
        }

        move_kind
    }

    fn get_game_state(&self, board: &Board) -> GameState {
        todo!()
    }
}

fn get_in_check(board: &Board, color: PieceColor) -> bool {
    let color = color.opposite();
    let mut in_check = false;
    for x in 0..8 {
        for y in 0..8 {
            let position = Position { x, y };
            if position.validate_bounds() {
                if board.get_piece(&position).color == color {
                    let moves = get_moves_internal(board, position, true);
                    for mov in moves {
                        if board.get_piece(&mov).kind == PieceKind::King {
                            in_check = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    in_check
}

fn is_check_after_move(board: &Board, from: Position, to: Position) -> bool {
    let mut board = board.clone();
    board.move_piece(&from, &to);
    get_in_check(&board, board.turn)
}

fn get_moves_internal(board: &Board, position: Position, skip_check: bool) -> Vec<Position> {
    match board.get_piece(&position).kind {
        PieceKind::Pawn => get_moves_pawn(board, position, skip_check),
        PieceKind::Knight => get_moves_knight(board, position, skip_check),
        PieceKind::Bishop => get_moves_bishop(board, position, skip_check),
        PieceKind::Rook => get_moves_rook(board, position, skip_check),
        PieceKind::Queen => get_moves_queen(board, position, skip_check),
        PieceKind::King => get_moves_king(board, position, skip_check),
        PieceKind::None => Vec::new(),
    }
}

fn get_moves_pawn(board: &Board, position: Position, skip_check: bool) -> Vec<Position> {
    let mut moves: Vec<Position> = Vec::new();

    let direction = match board.get_piece(&position).color {
        PieceColor::White => 1,
        PieceColor::Black => -1,
        _ => 0,
    };

    let next_move = position.add_delta((0, direction));
    if next_move.validate_bounds() {
        if board.get_piece(&next_move).kind == PieceKind::None {
            if skip_check || !is_check_after_move(board, position, next_move) {
                moves.push(next_move);
            }

            let begin_y = match board.get_piece(&position).color {
                PieceColor::White => 1,
                PieceColor::Black => 6,
                _ => 0,
            };

            if position.y == begin_y {
                let next_move = next_move.add_delta((0, direction));
                if next_move.validate_bounds() {
                    if board.get_piece(&next_move).kind == PieceKind::None {
                        if skip_check || !is_check_after_move(board, position, next_move) {
                            moves.push(next_move);
                        }
                    }
                }
            }
        }
    }

    for delta in vec![(1, direction), (-1, direction)] {
        let next_move = position.add_delta(delta);
        if next_move.validate_bounds() {
            if board.get_piece(&next_move).color == board.get_piece(&position).color.opposite() {
                if skip_check || !is_check_after_move(board, position, next_move) {
                    moves.push(next_move);
                }
            } else {
                // check for en passant
                let en_passant = board.en_passant;
                if en_passant.is_some() {
                    let en_passant = en_passant.unwrap();
                    if next_move.compare(&en_passant) {
                        if skip_check || !is_check_after_move(board, position, next_move) {
                            moves.push(next_move);
                        }
                    }
                }
            }
        }
    }

    moves
}

fn get_moves_knight(board: &Board, position: Position, skip_check: bool) -> Vec<Position> {
    let mut moves: Vec<Position> = Vec::new();

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

    for delta in deltas {
        let next_move = position.add_delta(delta);
        if next_move.validate_bounds() {
            if board.get_piece(&next_move).color != board.get_piece(&position).color {
                if skip_check || !is_check_after_move(board, position, next_move) {
                    moves.push(next_move);
                }
            }
        }
    }

    moves
}

fn get_moves_scanline(
    board: &Board,
    position: Position,
    skip_check: bool,
    deltas: Vec<(i8, i8)>,
) -> Vec<Position> {
    let mut moves: Vec<Position> = Vec::new();

    for delta in deltas {
        for i in 1..8 {
            let next_move = position.add_delta((delta.0 * i, delta.1 * i));
            if next_move.validate_bounds() {
                if board.get_piece(&next_move).color == board.get_piece(&position).color {
                    break;
                } else {
                    if skip_check || !is_check_after_move(board, position, next_move) {
                        moves.push(next_move);
                    }
                    if board.get_piece(&next_move).color != PieceColor::None {
                        break;
                    }
                }
            }
        }
    }

    moves
}

fn get_moves_bishop(board: &Board, position: Position, skip_check: bool) -> Vec<Position> {
    let deltas = vec![(1, 1), (1, -1), (-1, 1), (-1, -1)];
    get_moves_scanline(board, position, skip_check, deltas)
}

fn get_moves_rook(board: &Board, position: Position, skip_check: bool) -> Vec<Position> {
    let deltas = vec![(1, 0), (0, 1), (-1, 0), (0, -1)];
    get_moves_scanline(board, position, skip_check, deltas)
}

fn get_moves_queen(board: &Board, position: Position, skip_check: bool) -> Vec<Position> {
    let deltas = vec![
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
    ];
    get_moves_scanline(board, position, skip_check, deltas)
}

fn get_moves_king(board: &Board, position: Position, skip_check: bool) -> Vec<Position> {
    let mut moves: Vec<Position> = Vec::new();

    let deltas = vec![
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
    ];

    for delta in deltas {
        let next_move = position.add_delta(delta);
        if next_move.validate_bounds() {
            if board.get_piece(&next_move).color != board.get_piece(&position).color {
                if skip_check || !is_check_after_move(board, position, next_move) {
                    moves.push(next_move);
                }
            }
        }
    }

    // castling: (white_king, white_queen, black_king, black_queen)

    let castles = board.castles;

    let king_side = match board.get_piece(&position).color {
        PieceColor::White => castles.0,
        PieceColor::Black => castles.2,
        _ => false,
    };

    let queen_side = match board.get_piece(&position).color {
        PieceColor::White => castles.1,
        PieceColor::Black => castles.3,
        _ => false,
    };

    if king_side {
        let mut can_castle = true;
        for i in 1..3 {
            let next_move = position.add_delta((i, 0));
            if next_move.validate_bounds() {
                if board.get_piece(&next_move).kind != PieceKind::None {
                    can_castle = false;
                    break;
                }
            } else {
                can_castle = false;
                break;
            }
        }
        if can_castle {
            let next_move = position.add_delta((2, 0));
            if skip_check || !is_check_after_move(board, position, next_move) {
                moves.push(next_move);
            }
        }
    }

    if queen_side {
        let mut can_castle = true;
        for i in 1..4 {
            let next_move = position.add_delta((-i, 0));
            if next_move.validate_bounds() {
                if board.get_piece(&next_move).kind != PieceKind::None {
                    can_castle = false;
                    break;
                }
            } else {
                can_castle = false;
                break;
            }
        }
        if can_castle {
            let next_move = position.add_delta((-2, 0));
            if skip_check || !is_check_after_move(board, position, next_move) {
                moves.push(next_move);
            }
        }
    }

    moves
}
