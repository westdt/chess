import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue"
import { error, warn, info, debug, trace, setupBoard} from "./utils.ts"

import { invoke } from "@tauri-apps/api";
import { emit, listen } from '@tauri-apps/api/event'

info("Starting Vue app...");

let app = createApp(App);
app.mount("#app");

//document.addEventListener('contextmenu', event => event.preventDefault());

let setup = await setupBoard();

if (!setup) {
	error("Error setting up board");
} else { 
	info("Board setup successfully");
}

while (1) {
	const eventCreateElement = await listen('create-element', (event) => {
		debug("received create-element event from Rust");
		let id = event.payload[0];
		let kind = event.payload[1];

		debug("received create-element id: " + id + " kind: " + kind + " from Rust");

		//trace("received create-element id: " + id + " kind: " + kind + " from Rust");

		//let element = document.getElementById("rust" + payload[1]);
	});

	eventCreateElement();
}