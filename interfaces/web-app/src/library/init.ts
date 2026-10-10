import { sync } from "./sync.svelte.ts";
import { collection } from "./collection.svelte.ts";
import { prewarmer } from "./prewarmer.svelte.ts";
import { updateConfig } from "../config.svelte.ts";
import { applyColors } from "../colors.svelte.ts";
import appConfig from "../../config.json";

export function initApp() {
  prewarmer;

  sync.connect();

  collection.config = { ...collection.config, ...appConfig };
  updateConfig(appConfig);
  applyColors(appConfig);
}
