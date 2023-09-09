use tauri::Window;

use crate::bot::ChessBot;

use super::{Board, Engine, GameState, Move, Piece, PieceColor, PieceKind, Position};

pub(crate) struct EngineAlpha {}

impl Engine for EngineAlpha {
    fn new() -> Self {
        return EngineAlpha {};
    }

    fn get_moves(&self, position: &Position, board: &Board) -> Vec<Position> {
        get_moves_internal(self, board, position, true)
    }

    fn bot_move(&self, window: &Window, bot: &dyn ChessBot, board: &Board) -> Move {
        bot.get_move(board)
    }

    fn execute_move(&self, mov: &Move, board: &Board) -> Option<Board> {
        let to = &mov.to;
        let from = &mov.from;
        let piece = board.get_piece(from);
        if piece.color != board.turn {
            // invalid move, wrong turn
            return None;
        }

        let mut new_board = board.clone();
        let moves = get_moves_internal(self, board, &from, true);
        let mut valid = false;

        for position in moves.iter() {
            if position.compare(to) {
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
            if from.y == 1 || from.y == 6 {
                //check en passant

                if piece.color == PieceColor::White {
                    if from.y == 1 && to.y == 3 {
                        new_board.en_passant = Some(Position::from_x_y(to.x, 2));
                    }
                } else {
                    if from.y == 6 && to.y == 4 {
                        new_board.en_passant = Some(Position::from_x_y(to.x, 5));
                    }
                }
            } else {
                // check if piece killed is en passant
                let dir = match piece.color {
                    PieceColor::White => 1,
                    PieceColor::Black => -1,
                    PieceColor::None => 0,
                    PieceColor::Both => 0,
                };
                match new_board.en_passant {
                    Some(en_passant) => {
                        if en_passant.compare(to) {
                            // en passant!
                            let mut killed = en_passant;
                            killed = killed.mov(Position::from_x_y(0, -dir)).unwrap();
                            new_board.remove_piece(&killed);
                        }
                    }
                    None => {}
                }
            }

            new_board.en_passant = None;
            new_board.move_piece(from, to);
        } else {
            new_board.en_passant = None;
            if piece.kind == PieceKind::King {
                // check if move is a castling move or if it is just a normal move
                if self.check_castle(board, *from, *to) {
                    // YES! it is a castling move

                    // now we need to figure out which rook is being moved

                    let side = match to.x {
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
                        "queenside" => Position::from_x_y(3, to.y),
                        "kingside" => Position::from_x_y(5, to.y),
                        _ => Position::none(),
                    };

                    let king_new_position = match side {
                        "queenside" => Position::from_x_y(2, to.y),
                        "kingside" => Position::from_x_y(6, to.y),
                        _ => Position::none(),
                    };

                    new_board.move_piece(to, &rook_new_position);
                    new_board.move_piece(from, &king_new_position);
                } else {
                    new_board.move_piece(from, to);
                }
            } else if piece.kind == PieceKind::Rook {
                // mostly normal, just need to make sure that castling is disabled
                let side = match to.x {
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

                new_board.move_piece(from, to);
            } else {
                new_board.move_piece(from, to);
            }
        }

        new_board.turn = new_board.turn.opposite();
        new_board.selected_piece = None;
        Some(new_board)
    }

    fn validate_move(&self, mov: &Move, board: &Board) -> Option<Move> {
        // basically just check if the move is in the list of moves
        let moves = get_moves_internal(self, board, &mov.from, true);
        for i in 0..moves.len() {
            if moves[i].compare(&mov.to) {
                return Some(mov.clone());
            }
        }

        None
    }

    fn check_in_check(&self, color: PieceColor, board: &Board) -> bool {
        // check if king is in check
        let mut king_position = None;
        for i in 0..63 {
            let i = i as i8;
            let position = Position::from_index(i);
            if board.get_piece(&position).color == color
                && board.get_piece(&position).kind == PieceKind::King
            {
                king_position = Some(position);
                break;
            }
        }

        if let Some(king_position) = king_position {
            for i in 0..63 {
                let i = i as i8;
                let position = Position::from_index(i);
                if board.get_piece(&position).color != color {
                    let moves = get_moves_internal(self, board, &position, false);
                    for j in 0..moves.len() {
                        if moves[j].compare(&king_position) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn get_game_state(&self, board: &Board) -> GameState {
        let mut white_in_check = false;
        let mut black_in_check = false;

        let mut white_in_checkmate = true;
        let mut black_in_checkmate = true;

        let mut stalemate = true;

        for i in 0..63 {
            let position = Position::from_index(i);
            let piece = board.get_piece(&position);
            if piece.color == PieceColor::White {
                if piece.kind == PieceKind::King {
                    if self.check_in_check(PieceColor::White, board) {
                        white_in_check = true;
                        stalemate = false;
                    }
                } else {
                    let moves = get_moves_internal(self, board, &position, true);
                    if moves.len() > 0 {
                        white_in_checkmate = false;
                        stalemate = false;
                    }
                }
            } else if piece.color == PieceColor::Black {
                if piece.kind == PieceKind::King {
                    if self.check_in_check(PieceColor::Black, board) {
                        black_in_check = true;
                        stalemate = false;
                    }
                } else {
                    let moves = get_moves_internal(self, board, &position, true);
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

    fn check_castle(&self, board: &Board, king: Position, rook: Position) -> bool {
        let king_piece = board.get_piece(&king);
        let rook_piece = board.get_piece(&rook);

        let color = king_piece.color;

        if (rook.x != 0 && rook.x != 7) || (rook.y != 0 && rook.y != 7) {
            return false;
        }

        if (color == PieceColor::White) {
            if (board.white_castle_kingside || board.white_castle_queenside) {
                if (king.y == rook.y) {
                    if (king.x < rook.x) {
                        if (board.white_castle_kingside) {
                            return true;
                        }
                    } else {
                        if (board.white_castle_queenside) {
                            return true;
                        }
                    }
                }
            } else {
            }
        } else {
            if (board.black_castle_kingside || board.black_castle_queenside) {
                if (king.y == rook.y) {
                    if (king.x < rook.x) {
                        if (board.black_castle_kingside) {
                            return true;
                        }
                    } else {
                        if (board.black_castle_queenside) {
                            return true;
                        }
                    }
                }
            } else {
            }
        }
        false
    }
}

fn get_moves_internal(
    engine: &EngineAlpha,
    board: &Board,
    position: &Position,
    check_check: bool,
) -> Vec<Position> {
    match board.get_piece(position).kind {
        PieceKind::Pawn => get_moves_pawn(engine, board, position, check_check),
        PieceKind::Knight => get_moves_knight(engine, board, position, check_check),
        PieceKind::Bishop => get_moves_bishop(engine, board, position, check_check),
        PieceKind::Rook => get_moves_rook(engine, board, position, check_check),
        PieceKind::Queen => get_moves_queen(engine, board, position, check_check),
        PieceKind::King => get_moves_king(engine, board, position, check_check),
        PieceKind::None => Vec::new(),
    }
}

fn get_moves_pawn(
    engine: &EngineAlpha,
    board: &Board,
    position: &Position,
    check_check: bool,
) -> Vec<Position> {
    let mut moves = Vec::new();
    let piece = board.get_piece(position);
    let direction = match piece.color {
        PieceColor::White => 1,
        PieceColor::Black => -1,
        PieceColor::None => {
            return moves;
        }
        PieceColor::Both => {
            return moves;
        }
    };

    // single move forward
    let mut mov = position.mov(Position::from_x_y(0, 1 * direction));
    match mov {
        Some(new_position) => {
            if board.get_piece(&new_position).color == PieceColor::None {
                let move_obj = Move::validate(*position, new_position);

                if move_obj.is_some() {
                    moves.push(new_position);

                    mov = new_position.mov(Position::from_x_y(0, 1 * direction));

                    // double move forward
                    if (piece.color == PieceColor::White && position.y == 1)
                        || (piece.color == PieceColor::Black && position.y == 6)
                    {
                        mov = position.mov(Position::from_x_y(0, 2 * direction));
                        match mov {
                            Some(new_position) => {
                                if board.get_piece(&new_position).color == PieceColor::None {
                                    let move_obj = Move::validate(*position, new_position);
                                    if move_obj.is_some() {
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
    mov = position.mov(Position::from_x_y(-1, 1 * direction));
    match mov {
        Some(new_position) => {
            if board.get_piece(&new_position).color != piece.color
                && board.get_piece(&new_position).color != PieceColor::None
            {
                let move_obj = Move::validate(*position, new_position);
                if move_obj.is_some() {
                    moves.push(new_position);
                }
            }

            // en passant
            match board.en_passant {
                Some(en_passant) => {
                    if en_passant.compare(&Position::from_x_y(new_position.x, new_position.y)) {
                        let move_obj = Move::validate(*position, new_position);
                        if move_obj.is_some() {
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
    mov = position.mov(Position::from_x_y(1, 1 * direction));
    match mov {
        Some(new_position) => {
            if board.get_piece(&new_position).color != piece.color
                && board.get_piece(&new_position).color != PieceColor::None
            {
                let move_obj = Move::validate(*position, new_position);
                if move_obj.is_some() {
                    moves.push(new_position);
                }

                // en passant
                match board.en_passant {
                    Some(en_passant) => {
                        if en_passant.compare(&Position::from_x_y(new_position.x, new_position.y)) {
                            let move_obj = Move::validate(*position, new_position);
                            if move_obj.is_some() {
                                moves.push(new_position);
                            }
                        }
                    }
                    None => {}
                }
            }
        }
        None => {}
    }

    moves
}

fn get_moves_knight(
    engine: &EngineAlpha,
    board: &Board,
    position: &Position,
    check_check: bool,
) -> Vec<Position> {
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
        let mov = position.mov(Position::from_x_y(delta.0, delta.1));
        match mov {
            Some(new_position) => {
                if board.get_piece(&new_position).color != board.get_piece(position).color {
                    let move_obj = Move::validate(*position, new_position);
                    if move_obj.is_some() {
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
    engine: &EngineAlpha,
    board: &Board,
    position: &Position,
    deltas: Vec<(i8, i8)>,
    check_check: bool,
) -> Vec<Position> {
    let mut moves = Vec::new();

    for delta in deltas.iter() {
        for distance in 1..8 {
            let mov = position.mov(Position::from_x_y(delta.0 * distance, delta.1 * distance));
            match mov {
                Some(new_position) => {
                    if board.get_piece(&new_position).color == PieceColor::None {
                        let move_obj = Move::validate(*position, new_position);
                        if move_obj.is_some() {
                            moves.push(new_position);
                        }
                    } else if board.get_piece(&new_position).color
                        == board.get_piece(position).color.opposite()
                    {
                        let move_obj = Move::validate(*position, new_position);
                        if move_obj.is_some() {
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

fn get_moves_bishop(
    engine: &EngineAlpha,
    board: &Board,
    position: &Position,
    check_check: bool,
) -> Vec<Position> {
    let deltas = vec![(1, 1), (1, -1), (-1, -1), (-1, 1)];

    get_moves_scanline(engine, board, position, deltas, check_check)
}

fn get_moves_rook(
    engine: &EngineAlpha,
    board: &Board,
    position: &Position,
    check_check: bool,
) -> Vec<Position> {
    let deltas = vec![(1, 0), (0, 1), (-1, 0), (0, -1)];

    get_moves_scanline(engine, board, position, deltas, check_check)
}

fn get_moves_queen(
    engine: &EngineAlpha,
    board: &Board,
    position: &Position,
    check_check: bool,
) -> Vec<Position> {
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

    get_moves_scanline(engine, board, position, deltas, check_check)
}

fn get_moves_king(
    engine: &EngineAlpha,
    board: &Board,
    position: &Position,
    check_check: bool,
) -> Vec<Position> {
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
        let mov = position.mov(Position::from_x_y(delta.0, delta.1));
        match mov {
            Some(new_position) => {
                if board.get_piece(&new_position).color != board.get_piece(position).color {
                    let move_obj = Move::validate(*position, new_position);
                    if move_obj.is_some() {
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
    if board.get_piece(position).color == PieceColor::White {
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
            match position.mov(Position::from_x_y(-x_delta, 0)) {
                Some(position) => new_position = position,
                None => break,
            }

            if board.get_piece(&new_position).color != PieceColor::None {
                valid = false;
                break;
            }

            let move_obj = Move::validate(*position, new_position);

            // validate check
            if (move_obj.is_some()) {
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            moves.push(Position::from_x_y(0, position.y));
        }
    }

    if castle_kingside {
        let mut valid = true;
        for x_delta in 1..3 {
            let mut new_position;
            match position.mov(Position::from_x_y(x_delta, 0)) {
                Some(position) => new_position = position,
                None => break,
            }

            if board.get_piece(&new_position).color != PieceColor::None {
                valid = false;
                break;
            }

            let move_obj = Move::validate(*position, new_position);

            // validate check
            if (move_obj.is_some()) {
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            moves.push(Position::from_x_y(7, position.y));
        }
    }

    moves
}
