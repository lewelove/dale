import appConfig from "../config.json";

export interface PaletteConfig {
  ok100: string;
  ok200: string;
  ok300: string;
  ok400: string;
  ok500: string;
}

export const colorsState = $state<{
  palette: PaletteConfig;
}>({
  palette: { ...appConfig.palette }
});

let probeEl: HTMLSpanElement | null = null;

function getProbeElement(): HTMLSpanElement | null {
  if (!probeEl && typeof document !== "undefined") {
    probeEl = document.createElement("span");
    probeEl.style.display = "none";
    document.head.appendChild(probeEl);
  }
  return probeEl;
}

export function parseToOklchChannels(colorStr: string): string {
  if (!colorStr) return "1.00 0 0";
  const el = getProbeElement();
  if (!el) return "1.00 0 0";
  el.style.color = "";
  el.style.color = `oklch(from ${colorStr} l c h)`;
  const match = getComputedStyle(el).color.match(/oklch\(([^/)]+)/i);
  return match ? match[1].trim() : "1.00 0 0";
}

export function deriveColorTokens(baseColor: string): Record<string, string> {
  const channels = parseToOklchChannels(baseColor);
  return {
    "--text-main": `oklch(${channels} / 1)`,
    "--text-muted": `oklch(${channels} / 0.66)`,
    "--text-subtle": `oklch(${channels} / 0.50)`,
    "--border-muted": `oklch(${channels} / 0.04)`,
    "--border-subtle": `oklch(${channels} / 0.02)`,
    "--bg-surface-hover": `oklch(${channels} / 0.05)`,
    "--bg-surface-active": `oklch(${channels} / 0.08)`,
    "--bg-item-inactive": `oklch(${channels} / 0)`,
    "--bg-item-hover": `oklch(${channels} / 0.04)`,
    "--bg-item-active": `oklch(${channels} / 0.08)`,
    "--bg-button-inactive": `oklch(${channels} / 0.08)`,
    "--bg-button-hover": `oklch(${channels} / 0.12)`,
    "--bg-button-active": `oklch(${channels} / 0.16)`
  };
}

export function applyColors(configData: any) {
  if (!configData) return;

  if (configData.palette) {
    Object.assign(colorsState.palette, configData.palette);
  }

  if (typeof document === "undefined") return;
  const root = document.documentElement;

  for (const [key, val] of Object.entries(colorsState.palette)) {
    root.style.setProperty(`--color-${key}`, val);
  }
  root.style.setProperty(`--color-foreground`, colorsState.palette.ok100);
  root.style.setProperty(`--color-background`, colorsState.palette.ok300);

  const tokens = deriveColorTokens(colorsState.palette.ok100);
  for (const [prop, val] of Object.entries(tokens)) {
    root.style.setProperty(prop, val);
  }
}
