<script setup lang="ts">
import Square from "./Square.vue"
</script>

<template>
	<div id="grid-container">
		<Square v-for="i in 64" :key="i" :color="(i + (Math.floor((i - 1) / 8))) % 2 == 0 ? 'black' : 'white'"/>
		<Square :color="'highlight'" id="highlight"/> <!-- This is the highlight square -->
	</div>

	
</template>

<style>
img {
	display: block;
	position: absolute;
	left: -64px;
	top: -64px;
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

#selected {
	position: absolute;
	background-color: rgb(0, 191, 255);
	opacity: 0.6;
}

.allowed {
	border: 2px solid green;
}
</style>