// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, thread::sleep, time::SystemTime};

use chrono::Local;
use log::{LevelFilter};
use tauri::{Window};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;

mod chess_macros;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Piece {
    pid: i32,
    color: i32,
    kind: char,
    x: i32,
    y: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Board {
    pieces: Vec<Piece>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JsBoard {
	pieces: Vec<Piece>,
	highlight_blue: (i32, i32),
	highlights: Vec<(i32, i32)>
}

fn update_ui(window: &Window, board: &Board) {
	trace!("sending update-ui to JavaScript");
    window.emit("update-ui", board).unwrap();
}

#[tauri::command]
fn init_process(window: Window) {
    info!("Window is ready");

    let mut board = Board { pieces: Vec::new() };

    let mut pid = 0;

    for i in 0..8 {
        board.pieces.push(Piece {
            pid: pid,
            color: 0,
            kind: 'p',
            x: i,
            y: 1,
        });

        pid += 1;

        board.pieces.push(Piece {
            pid: pid,
            color: 1,
            kind: 'p',
            x: i,
            y: 6,
        });

        pid += 1;
    }

    board.pieces.push(Piece {
        pid: pid,
        color: 0,
        kind: 'r',
        x: 0,
        y: 0,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 0,
        kind: 'r',
        x: 7,
        y: 0,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 0,
        kind: 'n',
        x: 1,
        y: 0,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 0,
        kind: 'n',
        x: 6,
        y: 0,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 0,
        kind: 'b',
        x: 2,
        y: 0,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 0,
        kind: 'b',
        x: 5,
        y: 0,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 1,
        kind: 'r',
        x: 0,
        y: 7,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 1,
        kind: 'r',
        x: 7,
        y: 7,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 1,
        kind: 'n',
        x: 1,
        y: 7,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 1,
        kind: 'n',
        x: 6,
        y: 7,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 1,
        kind: 'b',
        x: 2,
        y: 7,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 1,
        kind: 'b',
        x: 5,
        y: 7,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 0,
        kind: 'q',
        x: 3,
        y: 0,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 0,
        kind: 'k',
        x: 4,
        y: 0,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 1,
        kind: 'q',
        x: 3,
        y: 7,
    });

    pid += 1;

    board.pieces.push(Piece {
        pid: pid,
        color: 1,
        kind: 'k',
        x: 4,
        y: 7,
    });

    pid += 1;

    std::thread::spawn(move || {
        loop {
            //info!("Updating UI");
            update_ui(&window, &board);
            sleep(std::time::Duration::from_millis(1000));
        }
    });
}

#[tauri::command]
fn request_allowed() {
	info!("Requesting permission to use camera");
}

#[tauri::command]
fn js_error(message: String) {
	error!(target: "JavaScript", "{}", message);
}

#[tauri::command]
fn js_warn(message: String) {
	warn!(target: "JavaScript", "{}", message);
}

#[tauri::command]
fn js_info(message: String) {
	info!(target: "JavaScript", "{}", message);
}

#[tauri::command]
fn js_debug(message: String) {
	debug!(target: "JavaScript", "{}", message);
}

#[tauri::command]
fn js_trace(message: String) {
	trace!(target: "JavaScript", "{}", message);
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let _tauri = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .with_colors(ColoredLevelConfig::default())
                .level(LevelFilter::Debug)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![js_error, js_warn, js_info, js_debug, js_trace, init_process, request_allowed])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
