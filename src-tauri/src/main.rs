// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{env, fmt::Display};

use log::LevelFilter;
use tauri::Window;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use ts_rs::TS;

mod engine;
mod utils;

use engine::{alpha::EngineAlpha, minimax::Minimax, Board, Bot, Engine, Position};

#[tauri::command]
fn bot_fight(window: Window, board: Board) {
    let mut new_board = board.clone();
    new_board.bot_color = new_board.turn;
    while true {
        new_board.bot_move();
        let _result = window.emit("update-board", &new_board);
    }
}

#[tauri::command]
fn bot_move(window: Window, board: Board) {
    let mut new_board = board.clone();
    new_board.bot_move();
    let _result = window.emit("update-board", &new_board);
}

#[tauri::command]
fn pick_square(window: Window, board: Board, position: String) -> bool {
    let position = Position::from_algebraic(position);
    let mut new_board = board.clone();

    debug!("{}", board);

    match board.selected {
        Some(selected) => {
            if board.get_piece(&selected).color == board.turn {
                let moves = board.get_moves(selected);
                let mut valid_move = false;
                for mov in moves {
                    if mov.compare(&position) {
                        valid_move = true;
                        break;
                    }
                }

                if valid_move {
                    board
                        .engine
                        .execute_move(&mut new_board, selected, position);
                    new_board.selected = None;
                    let _result = window.emit("set-highlights", "");
                } else {
                    if board.get_piece(&position).color == board.turn {
                        new_board.selected = Some(position);

                        let mut highlights: Vec<String> = Vec::new();
                        let moves = new_board.get_moves(position);
                        for mov in moves {
                            debug!("added {} to highlight", mov.to_algebraic());
                            highlights.push(mov.to_algebraic());
                        }
                        let _result = window.emit("set-highlights", highlights);
                    } else {
                        new_board.selected = None;
                        let _result = window.emit("set-highlights", "");
                    }
                }
            } else {
                new_board.selected = None;
                let _result = window.emit("set-highlights", "");
            }
        }
        None => {
            if new_board.get_piece(&position).color == board.turn {
                new_board.selected = Some(position);
                let mut highlights: Vec<String> = Vec::new();
                let moves = board.get_moves(position);
                for mov in moves {
                    debug!("added {} to highlight", mov.to_algebraic());
                    highlights.push(mov.to_algebraic());
                }
                let _result = window.emit("set-highlights", highlights);
            } else {
                new_board.selected = None;
                let _result = window.emit("set-highlights", "");
            }
        }
    }

    let _result = window.emit("update-board", new_board);

    true
}

#[tauri::command]
fn setup_board() -> Board {
    debug!("setup_board");
    let board: Board = Board::default();

    board
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
            bot_fight,
            bot_move,
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
