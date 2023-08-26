import { createApp } from "vue";
import App from "./App.vue"
import { error, warn, info, debug, trace } from "./utils.ts"
import * as Utils from "./utils.ts";
import * as Rust from "./rust.ts";

import { invoke } from "@tauri-apps/api";
import { emit, listen } from '@tauri-apps/api/event'

info("Starting Vue app...");

let app = createApp(App);
app.mount("#app");

//document.addEventListener('contextmenu', event => event.preventDefault());

Rust.execute(await invoke('setup_board'));
Rust.execute(await invoke("get_events"));

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