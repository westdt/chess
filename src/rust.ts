import * as Utils from "./utils.ts";

let board;

export function execute(this: any, events: any) {
	//Utils.debug("Executing events: " + JSON.stringify(events));

	let call = false;

	// execute a list of events
	for (let event of events) {
		Utils.trace("Called from Rust: " + event.function + JSON.stringify(event.arguments));
		try {
			call = eval(event.function).call(this, ...event.arguments);
		} catch (e) {
			Utils.error("Error calling function " + event.function + ": " + e + ". Ensure the function is in scope.");
			//return false;
		}
	}

	return call;
}

function removePiece(id: string) {
	let element = document.getElementById(id);
	if (element) {
		element.style.opacity = "0";
		element.id = "removed";
	}
}

function movePiece(previous: string, current: string) {
	let x_prev = Utils.fromAlgebraic(previous)[0];
	let y_prev = Utils.fromAlgebraic(previous)[1];
	let x_curr = Utils.fromAlgebraic(current)[0];
	let y_curr = Utils.fromAlgebraic(current)[1];

	let element = document.getElementById(previous);
	if (element) {
		element.style.left = x_curr * 64 + "px";
		element.style.bottom = y_curr * 64 + "px";
		element.id = current;
	}
}

function createPiece(id: string, svg_path: string, x_in_px: string, y_in_px: string) {
	let element = document.createElement("img");
	element.id = id;
	element.src = svg_path;
	element.style.position = "absolute";
	element.style.left = x_in_px;
	element.style.bottom = y_in_px;
	// set on top of other elements
	element.style.zIndex = "1";

	document.getElementById("grid-container")?.appendChild(element);

	return element;
}

function removeHighlights() {
	let old_elements = document.getElementsByClassName("highlighted");
	while (old_elements[0]) {
		old_elements[0].remove();
	}
	return true;
}

function createHighlight(x: string, y: string, kind: number) {
	if (kind > 0) {
		let element = document.createElement("div");
		element.style.position = "absolute";
		element.style.display = "block";
		element.style.left = x;
		element.style.bottom = y;
		//element.style.border = "2px solid green";
		element.style.backgroundColor = "rgb(255, 255, 0)";
		element.style.opacity = "0.3";
		element.style.width = "64px";
		element.style.height = "64px";
		element.className = "highlighted";

		document.getElementById("grid-container")?.appendChild(element);

		return element;
	} else {
		// blue highlight
		let element = document.createElement("div");
		element.style.position = "absolute";
		element.style.display = "block";
		element.style.left = x;
		element.style.bottom = y;
		//element.style.border = "2px solid green";
		element.style.backgroundColor = "rgb(0, 191, 255)";
		element.style.opacity = "0.6";
		element.style.width = "64px";
		element.style.height = "64px";
		element.className = "highlighted";

		document.getElementById("grid-container")?.appendChild(element);

		return element;
	}
}

function saveBoard(board: any) {
	return board;
}