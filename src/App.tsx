import { useEffect, useState } from "react";
import "./App.css";
import { debug, error, info, trace } from "./utils";
import Chessboard from "./Chessboard";

// App component
function App() {
	const [squareSize, setSquareSize] = useState(64);
	const [height, setHeight] = useState(window.innerHeight);
	const [width, setWidth] = useState(window.innerWidth);

	// run when the component is first rendered
	useEffect(() => {

		// set the square size
		setSquareSize(Math.min(window.innerWidth, window.innerHeight) / 8);

		// listen for window resize
		window.addEventListener("resize", () => {
			let prevSquareSize = squareSize;
			let newSquareSize = Math.min(window.innerWidth, window.innerHeight) / 8;

			// set the square size
			setSquareSize(newSquareSize);
			setHeight(window.innerHeight);
			setWidth(window.innerWidth);

			if (prevSquareSize !== newSquareSize) {
				trace("Square size changed to " + squareSize);
			}
		});
	}, []);

	return (
		<div className="App">
			<div data-tauri-drag-region className="drag"></div>
			<Chessboard windowHeight={height} windowWidth={width} />
			<div data-tauri-drag-region className="button-wrapper"></div>
		</div>
	);
}

export default App;
