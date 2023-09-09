use async_trait::async_trait;

use rand::Rng;
use tauri::Window;

use crate::{update_react_board, update_react_selection, Board, PieceKind, Position};

#[async_trait]
pub trait ChessAI {
    async fn get_move(&self, window: &Window, board: &Board) -> Board;
}

pub struct MinimaxAI {
    depth: i32,
}

#[async_trait]
impl ChessAI for MinimaxAI {
    async fn get_move(&self, window: &Window, board: &Board) -> Board {
        // use minimax to get the best move
        let mut board = board.clone();
        let mut moves: Vec<(Position, Position)> = Vec::new();
        let mut highest_score = -1000;
        let mut best_move = (Position::new(0), Position::new(0));

        for i in 0..63 {
            let i = i as i8;
            let position = Position::new(i);
            if board.get(&position).color == board.ai {
                let piece_moves = board.get_moves(&position, true);
                for j in 0..piece_moves.len() {
                    let mut new_board = board.clone();
                    let to = piece_moves[j];
                    let from = position;
                    new_board.mov(from, to);
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

        board.mov(best_move.0, best_move.1);
        board.turn = board.turn.opposite();
        board.selected_piece = None;
        board
    }
}

impl MinimaxAI {
    pub fn new(depth: i32) -> Self {
        MinimaxAI { depth }
    }

    fn evaluate(&self, board: &Board) -> i32 {
        let mut score: i32 = 0;
        for i in 0..63 {
            let i = i as i8;
            let position = Position::new(i);
            let piece = board.get(&position);
            if piece.color == board.ai {
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
        if depth == 0 {
            return self.evaluate(board);
        }

        if maximizing_player {
            let mut max_eval = -1000;

            for i in 0..63 {
                let i = i as i8;
                let position = Position::new(i);
                if board.get(&position).color == board.ai {
                    let piece_moves = board.get_moves(&position, true);
                    for j in 0..piece_moves.len() {
                        let mut new_board = board.clone();
                        let to = piece_moves[j];
                        let from = position;
                        new_board.mov(from, to);
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
                let position = Position::new(i);
                if board.get(&position).color == board.ai.opposite() {
                    let piece_moves = board.get_moves(&position, true);
                    for j in 0..piece_moves.len() {
                        let mut new_board = board.clone();
                        let to = piece_moves[j];
                        let from = position;
                        new_board.mov(from, to);
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

pub struct RandomAI {}

#[async_trait]
impl ChessAI for RandomAI {
    async fn get_move(&self, window: &Window, board: &Board) -> Board {
        let mut board = board.clone();
        let mut rng = rand::thread_rng();
        let mut moves: Vec<(Position, Position)> = Vec::new();

        for i in 0..63 {
            let i = i as i8;
            let position = Position::new(i);
            if board.get(&position).color == board.ai {
                let piece_moves = board.get_moves(&position, true);
                for j in 0..piece_moves.len() {
                    moves.push((position, piece_moves[j]));
                }
            }
        }

        let index = rng.gen_range(0..moves.len());

        let to = moves[index].1;
        let from = moves[index].0;

        board.mov(from, to);
        board.turn = board.turn.opposite();
        board.selected_piece = None;
        update_react_selection(window, &board, &"None".to_string());
        update_react_board(window, &board);

        board
    }
}

impl RandomAI {
    pub fn new() -> Self {
        RandomAI {}
    }
}
