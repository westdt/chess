import { useEffect, useState } from "react";
import { debug, info, trace } from "./utils";
import "./Chessboard.css";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { PieceKind } from "./bindings/PieceKind";
import { PieceColor } from "./bindings/PieceColor";
import { Board } from "./bindings/Board";

function pieceKindToChar(kind: PieceKind) {
	switch (kind) {
		case "Pawn":
			return "p";
		case "Knight":
			return "n";
		case "Bishop":
			return "b";
		case "Rook":
			return "r";
		case "Queen":
			return "q";
		case "King":
			return "k";
		default:
			return "x";
	}
}

function pieceColorToChar(color: PieceColor) {
	switch (color) {
		case "White":
			return "w";
		case "Black":
			return "b";
		default:
			return "x";
	}
}

function toAlgebraicNotation(square: number) {
	let col = square % 8;
	let row = Math.floor(square / 8);
	let colChar = String.fromCharCode(97 + col);
	let rowChar = String.fromCharCode(49 + row);
	return colChar + rowChar;
}

function fromAlgebraicNotation(square: string) {
	let col = square.charCodeAt(0) - 97;
	let row = square.charCodeAt(1) - 49;
	return col + (row * 8);
}

function Chessboard(props: { windowHeight: number, windowWidth: number }) {
	const [board, setBoard] = useState<Board>();
	const [highlights, setHighlights] = useState<string[]>([]);

	let squareSize = Math.min(props.windowHeight, props.windowWidth) / 8;

	// the board
	let squares = [];
	for (let i = 0; i < 64; i++) {
		let col = i % 8;
		let row = Math.floor(i / 8);
		let color = (col + row) % 2 === 0 ? "white" : "black";
		squares.push(<div className={"square " + color} style={{ "width": squareSize + "px", "height": squareSize + "px" }}></div>);
	}

	let pieces = [];
	let selectedHTML = <></>;
	let highlightsHTML = [];

	if (board) {
		for (let i = 0; i < 64; i++) {
			let col = i % 8;
			let row = Math.floor(i / 8);
			let piece = board.pieces[i];
			if (piece.kind != "None") {
				let color = pieceColorToChar(piece.color);
				let kind = pieceKindToChar(piece.kind);
				let svgpath = "pieces/default/" + color + kind + ".svg";
				//debug(svgpath);

				pieces.push(<img src={svgpath} className="piece" style={{ "width": squareSize + "px", "height": squareSize + "px", "top": (squareSize * (7 - row)) + "px", "left": (squareSize * col) + "px" }} />);
			}
		}

		let selected = board.selected;
		if (selected) {
			selectedHTML = <div className="selected-square" style={{ "width": squareSize + "px", "height": squareSize + "px", "top": (squareSize * (7 - selected.y)) + "px", "left": (squareSize * selected.x) + "px" }}></div>;
		}

		// highlights as algebraic notation
		for (let i = 0; i < highlights.length; i++) {
			let square = highlights[i];
			let squareIdx = fromAlgebraicNotation(square);
			let col = squareIdx % 8;
			let row = Math.floor(squareIdx / 8);
			highlightsHTML.push(<div className="highlighted-square" style={{ "width": squareSize + "px", "height": squareSize + "px", "top": (squareSize * (7 - row)) + "px", "left": (squareSize * col) + "px" }}></div>);
		}
	}

	// listen for click
	function handleClick(event: any) {
		let x_offset = (props.windowWidth - (squareSize * 8)) / 2;
		let y_offset = (props.windowHeight - (squareSize * 8)) / 2;
		let x = Math.floor((event.clientX - x_offset) / squareSize);
		let y = Math.floor((event.clientY - y_offset) / squareSize);

		if (x > 7 || y > 7 || x < 0 || y < 0) {
			return;
		}

		let positionAlg = toAlgebraicNotation(x + ((7 - y) * 8));
		if (board) {
			let b = board as Board;
			debug("<- pick_square");
			invoke("pick_square", { board: b, position: positionAlg }).then((nonsense) => {

			});
		}
	}

	useEffect(() => {
		invoke("setup_board").then((board) => {
			let board2 = board as Board;
			setBoard(board2);
		});

		listen("update-board", (event) => {
			debug("-> update-board");

			let board: Board = event.payload as Board;
			setBoard(board);

			if (board.turn == board.bot_color) {
				invoke("bot_move", { board: board });
			}
		});

		listen("set-highlights", (event) => {
			debug("-> set-highlights");

			let squares: string[] = event.payload as string[];
			setHighlights(squares);
		});

	}, []);

	return (
		<div className="chessboard" style={{ "top": "0" }} onClick={handleClick}>
			{squares}
			{pieces}
			{selectedHTML}
			{highlightsHTML}
		</ div>
	);
}

export default Chessboard;