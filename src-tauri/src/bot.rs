use async_trait::async_trait;

use rand::Rng;
use tauri::Window;

use crate::engine::{alpha::EngineAlpha, Board, Engine, Move, PieceKind, Position};

pub trait ChessBot {
    fn get_move(&self, board: &Board) -> Move;
}

pub struct MinimaxBot {
    depth: i32,
}

impl ChessBot for MinimaxBot {
    fn get_move(&self, board: &Board) -> Move {
        let engine = EngineAlpha::new();
        // use minimax to get the best move
        let mut board = board.clone();
        let mut moves: Vec<(Position, Position)> = Vec::new();
        let mut highest_score = -1000;
        let mut best_move = (Position::from_index(0), Position::from_index(0));

        for i in 0..63 {
            let i = i as i8;
            let position = Position::from_index(i);
            if board.get_piece(&position).color == board.bot {
                let piece_moves = engine.get_moves(&position, &board);
                for j in 0..piece_moves.len() {
                    let mut new_board = board.clone();
                    let to = piece_moves[j];
                    let from = position;
                    new_board.move_piece(&from, &to);
                    new_board.turn = new_board.turn.opposite();
                    new_board.selected_piece = None;
                    let score = self.minimax(&new_board, self.depth, false);
                    if score > highest_score {
                        highest_score = score;
                        best_move = (from, to);
                    }
                }
            }
        }

        Move::validate(best_move.0, best_move.1).unwrap()
    }
}

impl MinimaxBot {
    pub fn new(depth: i32) -> Self {
        MinimaxBot { depth }
    }

    fn evaluate(&self, board: &Board) -> i32 {
        let mut score: i32 = 0;
        for i in 0..63 {
            let i = i as i8;
            let position = Position::from_index(i);
            let piece = board.get_piece(&position);
            if piece.color == board.bot {
                match piece.kind {
                    PieceKind::Pawn => score += 1,
                    PieceKind::Knight => score += 3,
                    PieceKind::Bishop => score += 3,
                    PieceKind::Rook => score += 5,
                    PieceKind::Queen => score += 9,
                    PieceKind::King => score += 100,
                    PieceKind::None => (),
                }
            } else {
                match piece.kind {
                    PieceKind::Pawn => score -= 1,
                    PieceKind::Knight => score -= 3,
                    PieceKind::Bishop => score -= 3,
                    PieceKind::Rook => score -= 5,
                    PieceKind::Queen => score -= 9,
                    PieceKind::King => score -= 100,
                    PieceKind::None => (),
                }
            }
        }
        score
    }

    fn minimax(&self, board: &Board, depth: i32, maximizing_player: bool) -> i32 {
        let engine = EngineAlpha::new();
        if depth == 0 {
            return self.evaluate(board);
        }

        if maximizing_player {
            let mut max_eval = -1000;

            for i in 0..63 {
                let i = i as i8;
                let position = Position::from_index(i);
                if board.get_piece(&position).color == board.bot {
                    let piece_moves = engine.get_moves(&position, &board);
                    for j in 0..piece_moves.len() {
                        let mut new_board = board.clone();
                        let to = piece_moves[j];
                        let from = position;
                        new_board.move_piece(&from, &to);
                        new_board.turn = new_board.turn.opposite();
                        new_board.selected_piece = None;
                        let eval = self.minimax(&new_board, depth - 1, false);
                        max_eval = max_eval.max(eval);
                    }
                }
            }

            return max_eval;
        } else {
            let mut min_eval = 1000;

            for i in 0..63 {
                let i = i as i8;
                let position = Position::from_index(i);
                if board.get_piece(&position).color == board.bot.opposite() {
                    let piece_moves = engine.get_moves(&position, &board);
                    for j in 0..piece_moves.len() {
                        let mut new_board = board.clone();
                        let to = piece_moves[j];
                        let from = position;
                        new_board.move_piece(&from, &to);
                        new_board.turn = new_board.turn.opposite();
                        new_board.selected_piece = None;
                        let eval = self.minimax(&new_board, depth - 1, true);
                        min_eval = min_eval.min(eval);
                    }
                }
            }

            return min_eval;
        }
    }
}

pub struct RandomBot {}

impl ChessBot for RandomBot {
    fn get_move(&self, board: &Board) -> Move {
        let engine = EngineAlpha::new();
        let mut board = board.clone();
        let mut rng = rand::thread_rng();
        let mut moves: Vec<(Position, Position)> = Vec::new();

        for i in 0..63 {
            let i = i as i8;
            let position = Position::from_index(i);
            if board.get_piece(&position).color == board.bot {
                let piece_moves = engine.get_moves(&position, &board);
                for j in 0..piece_moves.len() {
                    moves.push((position, piece_moves[j]));
                }
            }
        }

        let index = rng.gen_range(0..moves.len());

        let to = moves[index].1;
        let from = moves[index].0;

        Move::validate(from, to).unwrap()
    }
}

impl RandomBot {
    pub fn new() -> Self {
        RandomBot {}
    }
}
