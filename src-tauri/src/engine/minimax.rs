use std::cmp::{max, min};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Board, Bot, Engine, PieceColor, PieceKind, Position};

use log::debug;

#[derive(TS, Clone, Serialize, Deserialize)]
#[ts(export, export_to = "bindings/Minimax.ts")]
pub struct Minimax {
    name: String,
}

impl Bot for Minimax {
    fn new() -> Self {
        Minimax {
            name: String::from("Minimax"),
        }
    }

    fn get_best_move(&self, board: &Board) -> (Position, Position) {
        let color = board.bot_color;
        let mut best_score = std::i32::MIN;
        let mut best_move = (Position::from_x_y(0, 0), Position::from_x_y(0, 0));
        for position_index in 0..64 {
            let position = Position::from_index(position_index);
            if board.get_piece(&position).color == color {
                let moves = board.engine.get_moves(board, position);
                for mov in moves {
                    let mut new_board = board.clone();
                    board.engine.execute_move(&mut new_board, position, mov);
                    let eval = minimax(&new_board, 2, std::i32::MIN, std::i32::MAX);
                    debug!(
                        "{} to {}: {}",
                        position.to_algebraic(),
                        mov.to_algebraic(),
                        eval
                    );
                    if eval > best_score {
                        best_score = eval;
                        best_move = (position, mov);
                    }
                }
            }
        }

        debug!(
            "best_move is {} to {}: value is {}",
            best_move.0.to_algebraic(),
            best_move.1.to_algebraic(),
            best_score
        );

        best_move
    }
}

fn evaluate(board: &Board, color: PieceColor) -> i32 {
    let mut score = 0;
    for position_index in 0..64 {
        let position = Position::from_index(position_index);
        let piece = board.get_piece(&position);
        if piece.color == color {
            match piece.kind {
                PieceKind::Pawn => score += 1,
                PieceKind::Knight => score += 3,
                PieceKind::Bishop => score += 3,
                PieceKind::Rook => score += 5,
                PieceKind::Queen => score += 9,
                PieceKind::King => score += 100,
                _ => {}
            }
        } else {
            match piece.kind {
                PieceKind::Pawn => score -= 1,
                PieceKind::Knight => score -= 3,
                PieceKind::Bishop => score -= 3,
                PieceKind::Rook => score -= 5,
                PieceKind::Queen => score -= 9,
                PieceKind::King => score -= 100,
                _ => {}
            }
        }
    }

    score
}

fn minimax(board: &Board, depth: u8, alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
		return evaluate(board, board.turn.opposite());
	}

	let mut alpha = alpha;
	let mut beta = beta;

	let mut mov_count = 0;

	if board.turn == board.bot_color {

		let mut score = std::i32::MIN;

		for i in 0..64 {
			let from = Position::from_index(i);

			let moves = board.engine.get_moves(board, from);

			for mov in moves {
				mov_count += 1;
				let mut new_board = board.clone();
				board.engine.execute_move(&mut new_board, from, mov);

				score = max(score, minimax(&new_board, depth - 1, alpha, beta));
				
				if score > beta {
					break;
				}
				alpha = max(alpha, score);
			}
		}

		if mov_count == 0 {
			return 10000;
		}

		return score;
	} else {

		let mut score = std::i32::MAX;

		for i in 0..64 {
			let from = Position::from_index(i);

			let moves = board.engine.get_moves(board, from);

			for mov in moves {
				mov_count += 1;
				let mut new_board = board.clone();
				board.engine.execute_move(&mut new_board, from, mov);

				score = min(score, minimax(&new_board, depth - 1, alpha, beta));
				
				if score < alpha {
					break;
				}
				beta = min(beta, score);
			}
		}

		if mov_count == 0 {
			return -10000;
		}

		return score;
	}
}
