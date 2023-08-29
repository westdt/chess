import { invoke } from "@tauri-apps/api/tauri";

export function toAlgebraic(location: number) {
	// convert a square to algebraic notation
	let x = location % 8;
	let y = Math.floor(location / 8);

	return String.fromCharCode(x + 97) + (y + 1);
}

export function fromAlgebraic(s: string) {
	// convert algebraic notation to a square
	let x = s.charCodeAt(0) - 97;
	let y = parseInt(s.charAt(1)) - 1;

	return x + 8 * y;
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