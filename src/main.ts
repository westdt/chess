import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue"
import { error, warn, info, debug, trace } from "./utils.ts"

import { invoke } from "@tauri-apps/api";

/*

color:
	0: white
	1: black

kind:
	p: pawn
	r: rook
	n: knight
	b: bishop
	q: queen
	k: king

x and y are the coordinates of the piece on the board. (0, 0) is the top left corner of the board.

*/

class Piece {
	pid: number;
	color: number;
	kind: string;
	x: number;
	y: number;
	element: any;

	constructor(pid: number, color: number, kind: string, x: number, y: number) {
		this.pid = pid;
		this.color = color;
		this.kind = kind;
		this.x = x;
		this.y = y;
		this.element = document.createElement("img");
	}
}

function updatePiece(piece: Piece) {
	let color = "w";
	if (piece.color == 1) {
		color = "b";
	}

	piece.element.src = "src/assets/pieces/" + color + piece.kind + ".svg";
	piece.element.style.left = piece.x * 64 + "px";
	piece.element.style.top = piece.y * 64 + "px";

	return piece.element;
}

function setLocation(x: number, y: number) {
	// set the location of the piece on the board
}

function movePiece() {

}

function addPiece() {
	// add a piece to the board

}



let app = createApp(App);
app.mount("#app");


let pieces: Piece[] = [];
let gridContainer = document.getElementById("grid-container");
document.addEventListener('contextmenu', event => event.preventDefault());

import { emit, listen } from '@tauri-apps/api/event'

invoke('init_process');

while (1) {
	const updateui = await listen('update-ui', (event) => {
		trace("received update-ui from Rust");

		let rustPieces = event.payload.pieces;

		for (let i = 0; i < rustPieces.length; i++) {
			let piece: Piece = pieces[i];

			if (i >= pieces.length) {
				trace("piece not present in JS. adding piece now");
				let piece = new Piece(
					rustPieces[i].pid, 
					rustPieces[i].color, 
					rustPieces[i].kind, 
					rustPieces[i].x, 
					rustPieces[i].y);
				updatePiece(piece);
				gridContainer?.appendChild(updatePiece(piece));

				pieces.push(piece);
			} else if (piece.pid != rustPieces[i].pid) {
				trace("PID mismatch! JS PID: " + piece.pid + " Rust PID: " + rustPieces[i].pid);
				trace("Removing piece.");
				pieces.splice(i, 1);
			}
		}
	});

	updateui();

	const sethighlights = await listen('set-highlights', (event) => {
		trace("received set-highlights from Rust");
	});

	sethighlights();
}