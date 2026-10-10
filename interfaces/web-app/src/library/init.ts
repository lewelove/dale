import { sync } from "./sync.svelte.ts";
import { collection } from "./collection.svelte.ts";
import { prewarmer } from "./prewarmer.svelte.ts";
import { updateConfig } from "../config.svelte.ts";
import { initColors } from "../colors.svelte.ts";
import appConfig from "../../config.json";

export function initApp() {
  prewarmer;

  initColors();
  sync.connect();

  collection.config = { ...collection.config, ...appConfig };
  updateConfig(appConfig);
}
