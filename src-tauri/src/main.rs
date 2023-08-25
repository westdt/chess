// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, thread::sleep};
use tauri::Window;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use utils::{fromAlgebraic, toAlgebraic};

mod utils;

const MASK_COLOR: u16 = 0b1000000000000000;
const MASK_SELECTED: u16 = 0b0100000000000000;
const MASK_TYPE: u16 = 0b0000011100000000;
const MASK_X: u16 = 0b0000000001110000;
const MASK_Y: u16 = 0b0000000000000111;

const COLOR_WHITE: u16 = 0b0000000000000000;
const COLOR_BLACK: u16 = 0b0001000000000000;

const TYPE_PAWN: u16 = 0b0000000000000000;
const TYPE_KNIGHT: u16 = 0b0000000100000000;
const TYPE_BISHOP: u16 = 0b0000001000000000;
const TYPE_ROOK: u16 = 0b0000001100000000;
const TYPE_QUEEN: u16 = 0b0000010000000000;
const TYPE_KING: u16 = 0b0000010100000000;

const SELECTED: u16 = 0b0100000000000000;
const NOT_SELECTED: u16 = 0b0000000000000000;

static mut SELECTED_LOCATION: u16 = 0b0000000000000000;

struct Piece {
    value: u16,
}

impl Piece {
    fn new(color: u16, kind: u16, x: u16, y: u16) -> Piece {
        return Piece {
            value: (color << 12) | (kind << 8) | (x << 4) | y,
        };
    }

    fn color(&self) -> u16 {
        return (self.value & MASK_COLOR) >> 12;
    }

    fn kind(&self) -> u16 {
        return (self.value & MASK_TYPE) >> 8;
    }

    fn x(&self) -> u16 {
        return (self.value & MASK_X) >> 4;
    }

    fn y(&self) -> u16 {
        return self.value & MASK_Y;
    }
}

#[derive(Serialize, Deserialize)]
struct Element {
    id: String,
    kind: String,
    attributes: HashMap<String, String>,
    properties: HashMap<String, String>,
}

impl Element {
    fn new(id: String, kind: String) -> Element {
        return Element {
            id: id,
            kind: kind,
            attributes: HashMap::new(),
            properties: HashMap::new(),
        };
    }

    fn set_attribute(&mut self, attribute: String, value: String) {
        self.attributes.insert(attribute, value);
    }

    fn set_property(&mut self, property: String, value: String) {
        self.properties.insert(property, value);
    }

    fn style(&self, style: String, value: String) {
        //self.window.emit("style-element", (self.id, style, value)).unwrap();
    }

    fn remove(&self) {
        //self.window.emit("remove-element", self.id).unwrap();
    }
}

fn get_moves() {

}

#[tauri::command]
fn select_move(location: String) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();

    let location = fromAlgebraic(location);
    let x = location.0;
    let y = location.1;

    let mut new_location = SELECTED | (x << 4) | y;
    if (new_location == unsafe { SELECTED_LOCATION }) {
        unsafe { SELECTED_LOCATION = NOT_SELECTED };
    } else {
        unsafe { SELECTED_LOCATION = new_location };

        let mut selected = Element::new(toAlgebraic((x, y)), "div".to_string());
        selected.set_property("left".to_string(), (x * 64).to_string() + "px");
        selected.set_property("top".to_string(), (y * 64).to_string() + "px");
        selected.set_attribute("class".to_string(), "highlighted".to_string());
        selected.set_property(
            "background-color".to_string(),
            "rgb(0, 191, 255)".to_string(),
        );
        elements.push(selected);


		/*
        for i in -1..2 {
            for j in -1..2 {
                if (i == 0 && j == 0) {
                    continue;
                }

                let mut new_x = x as i32 + i;
                let mut new_y = y as i32 + j;

                if (new_x < 0 || new_x > 7 || new_y < 0 || new_y > 7) {
                    continue;
                }

                let mut element = Element::new(100, "div".to_string());
                element.set_property("left".to_string(), (new_x * 64).to_string() + "px");
                element.set_property("top".to_string(), (new_y * 64).to_string() + "px");
                element.set_attribute("class".to_string(), "highlighted".to_string());
                element.set_property("border".to_string(), "2px solid green;".to_string());
                // make rgb a mustard yellow
                element.set_property(
                    "background-color".to_string(),
                    "rgb(255, 255, 0)".to_string(),
                );
                // set the opacity lower
                element.set_property("opacity".to_string(), "0.3".to_string());
                elements.push(element);
            }
        }*/
    }

    return elements;
}

#[tauri::command]
fn setup_board(window: Window) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();

    for i in 0..8 {
        let mut element = Element::new(toAlgebraic((i, 1)), "img".to_string());
        element.set_attribute("src".to_string(), "src/assets/pieces/wp.svg".to_string());
        element.set_property("left".to_string(), (i * 64).to_string() + "px");
        element.set_property("top".to_string(), "64px".to_string());
        elements.push(element);
    

        let mut element = Element::new(toAlgebraic((i, 6)), "img".to_string());
        element.set_attribute("src".to_string(), "src/assets/pieces/bp.svg".to_string());
        element.set_property("left".to_string(), (i * 64).to_string() + "px");
        element.set_property("top".to_string(), "384px".to_string());
        elements.push(element);
    
    }

    let mut element = Element::new(toAlgebraic((1, 0)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/wn.svg".to_string());
    element.set_property("left".to_string(), "64px".to_string());
    element.set_property("top".to_string(), "0px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((6, 0)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/wn.svg".to_string());
    element.set_property("left".to_string(), "384px".to_string());
    element.set_property("top".to_string(), "0px".to_string());
    elements.push(element);


	let mut element = Element::new(toAlgebraic((1, 7)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/bn.svg".to_string());
    element.set_property("left".to_string(), "64px".to_string());
    element.set_property("top".to_string(), "448px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((6, 7)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/bn.svg".to_string());
    element.set_property("left".to_string(), "384px".to_string());
    element.set_property("top".to_string(), "448px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((2, 0)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/wb.svg".to_string());
    element.set_property("left".to_string(), "128px".to_string());
    element.set_property("top".to_string(), "0px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((5, 0)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/wb.svg".to_string());
    element.set_property("left".to_string(), "320px".to_string());
    element.set_property("top".to_string(), "0px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((2, 7)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/bb.svg".to_string());
    element.set_property("left".to_string(), "128px".to_string());
    element.set_property("top".to_string(), "448px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((5, 7)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/bb.svg".to_string());
    element.set_property("left".to_string(), "320px".to_string());
    element.set_property("top".to_string(), "448px".to_string());
    elements.push(element);


	let mut element = Element::new(toAlgebraic((0, 0)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/wr.svg".to_string());
    element.set_property("left".to_string(), "0px".to_string());
    element.set_property("top".to_string(), "0px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((7, 0)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/wr.svg".to_string());
    element.set_property("left".to_string(), "448px".to_string());
    element.set_property("top".to_string(), "0px".to_string());
    elements.push(element);


	let mut element = Element::new(toAlgebraic((0, 7)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/br.svg".to_string());
    element.set_property("left".to_string(), "0px".to_string());
    element.set_property("top".to_string(), "448px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((7, 7)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/br.svg".to_string());
    element.set_property("left".to_string(), "448px".to_string());
    element.set_property("top".to_string(), "448px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((3, 0)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/wq.svg".to_string());
    element.set_property("left".to_string(), "256px".to_string());
    element.set_property("top".to_string(), "0px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((3, 7)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/bq.svg".to_string());
    element.set_property("left".to_string(), "256px".to_string());
    element.set_property("top".to_string(), "448px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((4, 0)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/wk.svg".to_string());
    element.set_property("left".to_string(), "192px".to_string());
    element.set_property("top".to_string(), "0px".to_string());
    elements.push(element);


    let mut element = Element::new(toAlgebraic((4, 7)), "img".to_string());
    element.set_attribute("src".to_string(), "src/assets/pieces/bk.svg".to_string());
    element.set_property("left".to_string(), "192px".to_string());
    element.set_property("top".to_string(), "448px".to_string());
    elements.push(element);


    return elements;
}

/*#[tauri::command]
fn init_process(window: Window) {
    info!("Event loop started.");

    std::thread::spawn(move || {
        loop {
            sleep(std::time::Duration::from_millis(1000));
        }
    });

}*/

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let _tauri = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .with_colors(ColoredLevelConfig::default())
                .level(LevelFilter::Trace)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            setup_board,
            select_move,
            utils::js_error,
            utils::js_warn,
            utils::js_info,
            utils::js_debug,
            utils::js_trace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
