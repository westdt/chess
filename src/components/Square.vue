<script setup lang="ts">
defineProps({
	color: String
});
</script>

<template>
	<div :class="[color]"></div>
</template>

<style>
* {
	width: 64px;
	height: 64px;
}

.black {
	background-color: #000000;
}

.white {
	background-color: #ffffff;
}

.highlight {
	display: block;
	position: absolute;
	background-color: gray;
	opacity: 0.2;
}
</style>