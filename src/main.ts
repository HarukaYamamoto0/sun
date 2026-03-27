import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface Config {
	autostart: boolean;
	step: number;
	resyncEnabled: boolean;
	resyncIntervalMs: number;
}

const slider = document.getElementById("slider") as HTMLInputElement;
const valueLabel = document.getElementById("value") as HTMLSpanElement;

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function updateSliderFill(value: number) {
	slider.style.background = `linear-gradient(to right, #f59e0b ${value}%, #2e2e2e ${value}%)`;
}

function applyConfig(config: Config) {
	slider.step = String(config.step);
}

async function init() {
	const [brightness, config] = await Promise.all([
		invoke<number>("get_brightness"),
		invoke<Config>("get_config"),
	]);

	applyConfig(config);
	slider.value = String(brightness);
	valueLabel.textContent = `${brightness}%`;
	updateSliderFill(brightness);

	await listen<Config>("config-updated", (event) => {
		applyConfig(event.payload);
	});
}

slider.addEventListener("input", () => {
	const val = Number(slider.value);
	valueLabel.textContent = `${val}%`;
	updateSliderFill(val);

	if (debounceTimer) clearTimeout(debounceTimer);
	debounceTimer = setTimeout(() => {
		invoke("set_brightness", { value: val });
	}, 300);
});

init();