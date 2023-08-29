import { invoke } from "@tauri-apps/api/tauri";
import { JsFunction } from "./bindings/JsFunction";
import { PieceColor } from "./bindings/PieceColor";
import { PieceType } from "./bindings/PieceType";
import * as Utils from "./utils";
import { GameState } from "./bindings/GameState";

let ORIGIN_X = 0;
let ORIGIN_Y = 0;
let SCALE = 64;

function createPiece(color: PieceColor, pieceType: PieceType, position: string) {
	let board = document.getElementById("board");

	let positionNumber = Utils.fromAlgebraic(position);

	Utils.debug(`Creating piece ${color} ${pieceType} at ${position} (${positionNumber})`);

	let c = color === "White" ? "w" : "b";
	let p = pieceType === "Pawn" ? "p" : pieceType === "Knight" ? "n" : pieceType === "Bishop" ? "b" : pieceType === "Rook" ? "r" : pieceType === "Queen" ? "q" : "k";

	if (board) {
		let piece = document.createElement("img");
		piece.id = position;
		piece.classList.add("piece");
		piece.style.height = `${SCALE}px`;
		piece.style.width = `${SCALE}px`;
		// piece.style.top = `${ORIGIN_Y + SCALE * Math.floor(position / 8)}px`;
		// place pieces from bottom to top
		piece.style.top = `${ORIGIN_Y + SCALE * (7 - Math.floor(positionNumber / 8))}px`;
		piece.style.left = `${ORIGIN_X + SCALE * (positionNumber % 8)}px`;
		piece.src = `./pieces/default/${c}${p}.svg`;
		board.appendChild(piece);
	}
}

function movePiece(from: string, to: string) {
	let board = document.getElementById("board");

	let positionNumber = Utils.fromAlgebraic(to);

	let toPiece = document.getElementById(to);
	let fromPiece = document.getElementById(from);

	if (board && toPiece) {
		toPiece.style.opacity = "0";
		toPiece.id = "removed";
	}

	if (board && fromPiece) {
		// place pieces from bottom to top
		fromPiece.style.top = `${ORIGIN_Y + SCALE * (7 - Math.floor(positionNumber / 8))}px`;
		fromPiece.style.left = `${ORIGIN_X + SCALE * (positionNumber % 8)}px`;
		fromPiece.id = to;
	}
}

function deletePiece(position: string) {
	let board = document.getElementById("board");

	if (board) {
		let piece = document.getElementById(position);
		if (piece) {
			board.removeChild(piece);
		}
	}
}

function promotePiece(from: string, to: string, pieceColor: PieceColor, pieceType: PieceType) {
	let board = document.getElementById("board");

	let positionNumber = Utils.fromAlgebraic(to);

	let toPiece = document.getElementById(to);
	let fromPiece = document.getElementById(from) as HTMLImageElement;

	if (board && toPiece) {
		toPiece.style.opacity = "0";
		toPiece.id = "removed";
	}

	if (board && fromPiece) {
		// place pieces from bottom to top
		fromPiece.style.top = `${ORIGIN_Y + SCALE * (7 - Math.floor(positionNumber / 8))}px`;
		fromPiece.style.left = `${ORIGIN_X + SCALE * (positionNumber % 8)}px`;
		fromPiece.id = to;

		let c = pieceColor === "White" ? "w" : "b";
		let p = pieceType === "Knight" ? "n" : pieceType === "Bishop" ? "b" : pieceType === "Rook" ? "r" : "q";

		fromPiece.src = `./pieces/default/${c}${p}.svg`;
	}
}

function createHighlight(kind: number, position: string) {
	let board = document.getElementById("board");

	let positionNumber = Utils.fromAlgebraic(position);

	if (board) {
		let highlight = document.createElement("div");
		highlight.classList.add("highlight");
		highlight.style.height = `${SCALE}px`;
		highlight.style.width = `${SCALE}px`;

		if (kind > 0) {
			highlight.style.backgroundColor = "rgb(0, 191, 255)";
			highlight.style.opacity = "0.5";
		} else {
			highlight.style.backgroundColor = "rgb(255, 255, 0)";
			highlight.style.opacity = "0.3";
		}

		// highlight.style.top = `${ORIGIN_Y + SCALE * Math.floor(position / 8)}px`;
		// place pieces from bottom to top
		highlight.style.top = `${ORIGIN_Y + SCALE * (7 - Math.floor(positionNumber / 8))}px`;
		highlight.style.left = `${ORIGIN_X + SCALE * (positionNumber % 8)}px`;
		board.appendChild(highlight);
	}
}


function deleteHighlights() {
	let board = document.getElementById("board");
	if (board) {
		let highlights = document.getElementsByClassName("highlight");
		while (highlights.length > 0) {
			board.removeChild(highlights[0]);
		}
	}
}

function gameOver(gameState: GameState) {
	switch (gameState) {
		case "WhiteCheckmate":
			Utils.info("White checkmate!");

			let board_ele = document.getElementById("board");
			let gameOver_ele = document.createElement("div");
			gameOver_ele.id = "gameover";
			gameOver_ele.style.opacity = "0";

			let message_ele = document.createElement("h1");
			message_ele.innerText = "White wins!";
			message_ele.style.opacity = "0";

			if (board_ele) {
				board_ele.appendChild(gameOver_ele);
				gameOver_ele.appendChild(message_ele);

				// fade in
				gameOver_ele.style.opacity = "1";
				message_ele.style.opacity = "1";
			}

			break;
		case "BlackCheckmate":
			Utils.info("Black checkmate!");

			let board = document.getElementById("board");
			let gameOver = document.createElement("div");
			gameOver.id = "gameover";
			gameOver.style.opacity = "0";

			let message = document.createElement("h1");
			message.innerText = "White wins!";
			message.style.opacity = "0";

			if (board) {
				board.appendChild(gameOver);
				gameOver.appendChild(message);

				// fade in
				gameOver.style.opacity = "1";
				message.style.opacity = "1";
			}

			break;
		case "Stalemate":
			Utils.info("Stalemate!");

			Utils.info("Black checkmate!");

			let board2 = document.getElementById("board");
			let gameOver2 = document.createElement("div");
			gameOver2.id = "gameover";
			gameOver2.style.opacity = "0";

			let message2 = document.createElement("h1");
			message2.innerText = "Stalemate!";
			message2.style.opacity = "0";

			if (board2) {
				board2.appendChild(gameOver2);
				gameOver2.appendChild(message2);

				// fade in
				gameOver2.style.opacity = "1";
				message2.style.opacity = "1";
			}

			break;
		default:
			break;
	}
}

function processRustRequest(jsfunction: JsFunction, args: any[]) {
	//Utils.info("Processing request: " + JSON.stringify(jsfunction));
	switch (jsfunction) {
		case "None":
			break;
		case "CreatePiece":
			createPiece(args[0], args[1], args[2]);
			break;
		case "MovePiece":
			movePiece(args[0], args[1]);
			break;
		case "DeletePiece":
			deletePiece(args[0]);
			break;
		case "PromotePiece":
			promotePiece(args[0], args[1], args[2], args[3]);
			break;
		case "DeleteBoard":
			//STATE_BOARD_SET_PIECES(pieces => []);
			break;
		case "CreateHighlight":
			createHighlight(args[0], args[1]);
			break;
		case "DeleteHighlights":
			deleteHighlights();
			break;
		case "CallChessBot":
			Utils.debug("Calling chess bot...");
			invoke("chess_bot").then((requests) => {
				processRustRequests(requests);
			});
			break;
		case "GameOver":
			gameOver(args[0]);
			break;
		default:
			break;
	}
}

function processRustRequests(requests: any) {
	Utils.debug("Processing requests...");
	for (let request of requests) {
		processRustRequest(request.jsfunction, request.args);
	}
}

window.addEventListener("DOMContentLoaded", () => {
	let board = document.getElementById("board");

	if (board) {
		// create a chessboard
		// loop through 64 squares and add them to the board in alternating colors	
		for (let i = 0; i < 64; i++) {
			let x = Math.floor(i / 8);
			const square = document.createElement("div");
			square.classList.add(((i + x) % 2) === 0 ? "white" : "black");
			square.style.height = `${SCALE}px`;
			square.style.width = `${SCALE}px`;
			board.appendChild(square);
		}

		board.addEventListener("click", (e) => {
			let x = e.clientX - ORIGIN_X;
			let y = e.clientY - ORIGIN_Y;
			let file = Math.floor(x / SCALE);
			let rank = 7 - Math.floor(y / SCALE);

			let value = file + rank * 8;
			let algebraic = Utils.toAlgebraic(value);

			invoke("select", { location: algebraic }).then((requests) => {
				processRustRequests(requests);
			});
		});
	}
});

invoke("setup").then((requests) => {
	processRustRequests(requests);
});