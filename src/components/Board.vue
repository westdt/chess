<script setup lang="ts">
import Square from "./Square.vue"
</script>

<template>
	<div id="grid-container">
		<Square v-for="i in 64" :key="i" :color="(i + (Math.floor((i - 1) / 8))) % 2 == 0 ? 'black' : 'white'" />
		<Square :color="'highlight'" id="highlight" /> <!-- This is the highlight square -->


	</div>
</template>

<style>
img {
	display: block;
	position: absolute;

	transition: all 0.2s ease-out;
}

body {
	padding: 0;
	margin: 0;

	overflow: hidden;
}

#grid-container {
	display: grid;
	grid-template-columns: auto auto auto auto auto auto auto auto;
}

.highlighted {
	position: absolute;
	opacity: 0.6;
}
</style>