// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{env, fmt::Display};

use engine::{alpha::EngineAlpha, Engine, Move, PieceKind, Position};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use tauri::{PhysicalSize, Window};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use ts_rs::TS;

use crate::{
    bot::{ChessBot, MinimaxBot, RandomBot},
    engine::{Board, GameState},
};

mod bot;
mod engine;
mod utils;

fn update_react_board(window: &Window, board: &Board) {
    trace!("<- update-board");
    let _update_board = window.emit("update-board", board);
}

fn update_react_selection(window: &Window, board: &Board) {
    /*
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
    */
}

async fn update_react_game_state(window: &Window, board: &Board) {
    trace!("<- game-state");
    let game_state = GameState::None;
    match game_state {
        GameState::Playing => {}
        GameState::WhiteInCheck => {
            let _update_game_state = window.emit("game-state", String::from("white-in-check"));
        }
        GameState::BlackInCheck => {
            let _update_game_state = window.emit("game-state", String::from("black-in-check"));
        }
        GameState::WhiteInCheckmate => {
            let _update_game_state = window.emit("game-state", String::from("white-in-checkmate"));
        }
        GameState::BlackInCheckmate => {
            let _update_game_state = window.emit("game-state", String::from("black-in-checkmate"));
        }
        GameState::Stalemate => {
            let _update_game_state = window.emit("game-state", String::from("stalemate"));
        }
        GameState::None => {}
    }
}

// Tauri command: setup_board
// returns the default chess board
#[tauri::command]
fn setup_board(window: Window) -> Board {
    trace!("-> setup_board");

    Board::default_board()
}

#[tauri::command]
async fn pick_square(window: Window, mut board: Board, square: String) {
    let engine = EngineAlpha::new();

    let position = Position::from_algebraic(square);

    match board.selected_piece {
        Some(selected_piece) => {
            if board.get_piece(&selected_piece).kind == PieceKind::King
                && engine.check_castle(&board, selected_piece, position)
            {
                match &Move::validate(selected_piece, position) {
                    Some(mov) => match engine.validate_move(mov, &board) {
                        Some(mov) => {
                            board = engine.execute_move(&mov, &board).unwrap();
                            board.turn = board.turn.opposite();
                            board.selected_piece = None;
                        }
                        None => {
                            board.selected_piece = None;
                        }
                    },
                    None => {
                        board.selected_piece = None;
                    }
                }
            } else {
                board.selected_piece = Some(position);
            }
        }
        None => {
            if board.get_piece(&position).color == board.turn {
                board.selected_piece = Some(position);
            }
        }
    }

    update_react_selection(&window, &board);
    update_react_board(&window, &board);
    update_react_game_state(&window, &board);

    let bot_move = engine.bot_move(&window, &MinimaxBot::new(3), &board);
    board = engine.execute_move(&bot_move, &board).unwrap();
    board.turn = board.turn.opposite();
    update_react_board(&window, &board);
    update_react_game_state(&window, &board);
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
