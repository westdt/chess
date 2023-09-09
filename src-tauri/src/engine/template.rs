use super::{Engine, Board, Position, Move, PieceColor, GameState};

pub(crate) struct EngineTemplate {}

impl Engine for EngineTemplate {
    fn new() -> Self {
        return EngineTemplate {};
    }

    fn get_moves(&self, position: &Position, board: &Board) -> Vec<Position> {
        todo!()
    }

    fn bot_move(&self, board: &Board) -> Board {
        todo!()
    }

    fn execute_move(&self, mov: &Move, board: &Board) -> Board {
        todo!()
    }

    fn validate_move(&self, mov: &Move, board: &Board) -> Option<Move> {
        todo!()
    }

    fn check_in_check(&self, color: PieceColor, board: &Board) -> bool {
        todo!()
    }

    fn get_game_state(&self, board: &Board) -> GameState {
        todo!()
    }

    fn check_castle(&self, king: Position, rook: Position) -> bool {
        todo!()
    }
}

impl EngineAlpha {
	
}