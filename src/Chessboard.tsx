import { useEffect, useState } from "react";
import { debug, info, trace } from "./utils";
import "./Chessboard.css";
import { invoke } from "@tauri-apps/api";
import { PieceColor } from "./bindings/PieceColor";
import { PieceKind } from "./bindings/PieceKind";
import { Board } from "./bindings/Board";
import { listen } from "@tauri-apps/api/event";
import { Piece } from "./bindings/Piece";

// utility function to convert an index (0-63) to algebraic notation
function fromIndexToAlgebraic(index: number): string {
	let x = index % 8;
	let y = Math.floor(index / 8);
	return toAlgebraic(x, y);
}

// utility function to convert algebraic notation to an x (0-7), y (0-7) coordinate
function toXY(position: string): [number, number] {
	return [position.charCodeAt(0) - 97, 8 - parseInt(position.charAt(1))];
}

// utility function to convert an x (0-7), y (0-7) coordinate to algebraic notation
function toAlgebraic(x: number, y: number): string {
	return String.fromCharCode(x + 97) + (8 - y);
}

function pieceColorToChar(color: PieceColor): string {
	if (color === "White") {
		return "w";
	} else if (color === "Black") {
		return "b";
	} else {
		return "x";
	}
}

function PieceKindToChar(type: PieceKind): string {
	if (type === "Pawn") {
		return "p";
	} else if (type === "Rook") {
		return "r";
	} else if (type === "Knight") {
		return "n";
	} else if (type === "Bishop") {
		return "b";
	} else if (type === "Queen") {
		return "q";
	} else if (type === "King") {
		return "k";
	} else {
		return "x";
	}
}

/*
 * Chessboard component
 *
 * Props:
 * squareSize: the size of each square in pixels
 * 
 * Events:
 * update-board: update the chessboard
 * set-highlights: set the highlighted squares
 * select-square: select a square
 * 
*/
function Chessboard(props: { squareSize: number, top: number }) {
	const [chessBoard, setChessBoard] = useState<Board>(null!);
	const [selectedSquare, setSelectedSquare] = useState<string>("");
	const [highlightedSquares, setHighlightedSquares] = useState<string[]>([]);
	const [theme, setTheme] = useState("default");


	// create the chessboard (64 squares, no pieces)
	let squares: any = [];
	for (let i = 0; i < 64; i++) {
		let color = (i + Math.floor(i / 8)) % 2 === 0 ? "white" : "black";
		squares.push(<div className={"square " + color} style={{ "width": props.squareSize + "px", "height": props.squareSize + "px" }}></div>);
	}

	let result = <></>;

	if (chessBoard) {
		// render the pieces on the chessboard
		let pieces = chessBoard.pieces.map((piece) => {
			trace("Piece: " + JSON.stringify(piece));

			// get piece data including position, type, color, and image path
			let position = piece.position;
			let algebraicPosition = fromIndexToAlgebraic(position.value);
			let [x, y] = toXY(algebraicPosition);

			if (piece.kind === "None") {
				return <></>;
			}

			let PieceKind = PieceKindToChar(piece.kind);
			let pieceColor = pieceColorToChar(piece.color);
			let pieceImagePath = "pieces/" + theme + "/" + pieceColor + PieceKind + ".svg";



			// return the piece as an img element with width and height equal to the square size
			// a1 is the lower left corner of the chessboard
			return <img src={pieceImagePath} id={"p" + piece.pid} className="piece" style={{ "width": props.squareSize + "px", "height": props.squareSize + "px", "position": "absolute", "left": x * props.squareSize + "px", "bottom": y * props.squareSize + "px" }} />;
		});

		// render the highlighted squares
		let highlightedSquaresElements = highlightedSquares.map((square) => {

			// get square position
			let [x, y] = toXY(square);

			// return the square as a div element with width and height equal to the square size
			// a1 is the lower left corner of the chessboard
			return <div className="highlighted-square" style={{ "width": props.squareSize + "px", "height": props.squareSize + "px", "position": "absolute", "left": x * props.squareSize + "px", "bottom": (7 - y) * props.squareSize + "px" }}></div>;
		});

		let selectedSquareElement = <></>
		// render the selected square
		if (selectedSquare !== null && selectedSquare !== "") {
			let [x, y] = toXY(selectedSquare);
			selectedSquareElement = <div className="selected-square" style={{ "width": props.squareSize + "px", "height": props.squareSize + "px", "position": "absolute", "left": x * props.squareSize + "px", "bottom": (7 - y) * props.squareSize + "px" }}></div>;
		}

		// set result
		result = <>{pieces}{highlightedSquaresElements}{selectedSquareElement}</>;
	}

	function handleClick(event: any) {
		let x = Math.floor(event.clientX / props.squareSize);
		let y = Math.floor(event.clientY / props.squareSize);

		if (x > 7 || y > 7 || x < 0 || y < 0) {
			return;
		}

		let position = toAlgebraic(x, y);
		debug("Clicked square: " + position);
		if (chessBoard) {
			invoke("pick_square", { board: chessBoard, square: position })
		} else {
			debug("Chessboard is null");
		}
	}

	// invoke setup_board from Tauri when the component is first rendered
	useEffect(() => {
		debug("useEffect called");

		listen("update-board", (event) => {
			debug("update-board event received");

			let board: Board = event.payload as Board;
			setChessBoard(board);
		});

		listen("set-highlights", (event) => {
			debug("set-highlights event received");

			let squares: string[] = event.payload as string[];
			setHighlightedSquares(squares);
		});

		listen("select-square", (event) => {
			debug("select-square event received");

			let square: string = event.payload as string;
			setSelectedSquare(square);
		});

		// invoke setup_board from Tauri
		invoke("setup_board").then((result) => {
			let board: Board = result as Board;

			// update the chessboard
			setChessBoard(board);
		});
	}, []);

	return (
		<div className="chessboard" style={{ "width": props.squareSize * 8 + "px", "height": props.squareSize * 8 + "px", "top": props.top + "px" }} onClick={handleClick}>
			{squares}
			{result}
		</div>
	);
}

export default Chessboard;