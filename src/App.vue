<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { info, error, toAlgebraic, fromAlgebraic, selectMove } from "./utils.ts"

import Board from "./components/Board.vue"
import { invoke } from "@tauri-apps/api";

import "./utils.ts"


let selectedPosition: string = "none";

function mouseEnter() {
	let highlight = document.getElementById("highlight");

	if (highlight == null) {
		error("Highlight square not found");
		return;
	}

	highlight.style.display = "block";
}

function mouseMove(event: MouseEvent) {
	let highlight = document.getElementById("highlight");

	if (highlight == null) {
		error("Highlight square not found");
		return;
	}

	highlight.style.display = "block";
	highlight.style.left = Math.floor(event.clientX / 64) * 64 + "px";
	highlight.style.top = Math.floor(event.clientY / 64) * 64 + "px";
}

function mouseLeave() {
	let highlight = document.getElementById("highlight");

	if (highlight == null) {
		error("Highlight square not found");
		return;
	}

	highlight.style.display = "none";
}

async function onClick(event: MouseEvent) {
	let x = Math.floor(event.clientX / 64);
	let y = Math.floor(event.clientY / 64);

	selectMove(x, y);

	/*
	let selected = document.getElementById("selected");

	if (selected == null) {
		selected = document.createElement("div");
		document.getElementById("grid-container")?.appendChild(selected);
	} else {
	}

	let normalX = Math.floor(event.clientX / 64);
	let normalY = Math.floor(event.clientY / 64);
	let previousX = selected.style.left;
	let previousY = selected.style.top;
	let newX = normalX * 64 + "px";
	let newY = normalY * 64 + "px";

	selected.style.left = newX;
	selected.style.top = newY;
	selected.style.width = "64px";
	selected.style.height = "64px";
	selected.id = "selected";

	selected.style.display = "block"

	if (previousX == newX && previousY == newY) {
		selected.style.display = "none";

		selected.style.left = -64 + "px";
		selected.style.top = -64 + "px";

		info("Deselected square.");

		selectedPosition = "none";
	} else {
		let algebraic = toAlgebraic(normalX, normalY);

	
		info("Selected square: " + algebraic);
		invoke("request_allowed", { from: algebraic });

		selectedPosition = algebraic.toString();
	}
	*/
}
</script>

<template>
	<div class="container" @mouseenter="mouseEnter" @mousemove="mouseMove" @mouseleave="mouseLeave" @click="onClick">
		<Board />

	</div>
</template>

<style>
* {
	-webkit-user-select: none;
	-webkit-touch-callout: none;
	-moz-user-select: none;
	-ms-user-select: none;
	user-select: none;
}

*, *:hover, *:active, *:focus {
	cursor: default;
}
</style>
