import { invoke } from "@tauri-apps/api";

function loopElements(elements: any) {
	for (let i = 0; i < elements.length; i++) {
		let id = elements[i].id;
		let kind = elements[i].kind;
		let attributes: Map<string, string> = new Map(Object.entries(elements[i].attributes));
		let properties: Map<string, string> = new Map(Object.entries(elements[i].properties));
		let element = document.createElement(kind);
		element.id = id;

		for (let [key, value] of attributes) {
			element.setAttribute(key, value);
		}

		for (let [key, value] of properties) {
			element.style[key] = value;
		}

		trace("Adding element: " + element.outerHTML);

		document.getElementById("grid-container")?.appendChild(element);
	}
}

export async function setupBoard() {
	try {
		let result = await invoke('setup_board');
		loopElements(result);
	} catch (e) {
		error("Error setting up board: " + e);
		return false;
	}

	return true;
}

export async function selectMove(x: number, y: number) {
	try {
		debug("Selecting move: " + toAlgebraic(x, y));
		let result = await invoke('select_move', { location: toAlgebraic(x, y) });
		let old_elements = document.getElementsByClassName("highlighted");
		while (old_elements[0]) {
			old_elements[0].remove();
		}

		loopElements(result);
	} catch (e) {
		error("Error selecting move: " + e);
		return false
	}

	return true;
}

export function toAlgebraic(x: number, y: number) {
	// convert a square to algebraic notation
	return String.fromCharCode(x + 97) + (y + 1);
}

export function fromAlgebraic(s: string) {
	// convert algebraic notation to a square
	return [s.charCodeAt(0) - 97, parseInt(s[1]) - 1];
}

export function error(message: string) {
	invoke("js_error", { message: message });
}

export function warn(message: string) {
	invoke("js_warn", { message: message });
}

export function info(message: string) {
	invoke("js_info", { message: message });
}

export function debug(message: string) {
	invoke("js_debug", { message: message });
}

export function trace(message: string) {
	invoke("js_trace", { message: message });
}