import { invoke } from "@tauri-apps/api";

import * as Rust from "./rust.ts"

export async function selectMove(oldX: number, oldY: number, x: number, y: number) {
	try {
		debug("Selecting move: " + toAlgebraic(x, y));
		let result = await invoke('select_move', { location: toAlgebraic(x, y) });
		Rust.execute(result);

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