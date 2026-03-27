import {invoke} from "@tauri-apps/api/core";

interface Config {
	autostart: boolean;
	step: number;
	resyncEnabled: boolean;
	resyncIntervalMs: number;
}

const autostartEl = document.getElementById("autostart") as HTMLInputElement;
const stepEl = document.getElementById("step") as HTMLSelectElement;
const resyncEl = document.getElementById("resync") as HTMLInputElement;
const resyncIntervalEl = document.getElementById("resync-interval") as HTMLSelectElement;
const resyncIntervalRow = document.getElementById("resync-interval-row") as HTMLDivElement;
const saveBtn = document.getElementById("save") as HTMLButtonElement;

async function init() {
	try {
		const config = await invoke<Config>("get_config");
		autostartEl.checked = config.autostart;
		stepEl.value = String(config.step);
		resyncEl.checked = config.resyncEnabled;
		resyncIntervalEl.value = String(config.resyncIntervalMs);
		toggleResyncInterval(config.resyncEnabled);
	} catch (e) {
		console.error("failed to load config:", e);
	}
}

function toggleResyncInterval(visible: boolean) {
	resyncIntervalRow.classList.toggle("visible", visible);
}

resyncEl.addEventListener("change", () => toggleResyncInterval(resyncEl.checked));

saveBtn.addEventListener("click", async () => {
	const config: Config = {
		autostart: autostartEl.checked,
		step: Number(stepEl.value),
		resyncEnabled: resyncEl.checked,
		resyncIntervalMs: Number(resyncIntervalEl.value),
	};
	await invoke("save_config", { newConfig: config });

	saveBtn.textContent = "Saved";
	saveBtn.style.background = "#16a34a";
	setTimeout(() => {
		saveBtn.textContent = "Save";
		saveBtn.style.background = "";
	}, 1500);
});

init();