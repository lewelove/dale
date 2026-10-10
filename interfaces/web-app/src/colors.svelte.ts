export const canvasColors = $state({
  bg: "oklch(0.29 0 0)",
  title: "oklch(1 0 0)",
  artist: "oklch(1 0 0 / 0.7)",
  shadowOuter: "oklch(0 0 0 / 0.3)",
  shadowInner: "oklch(0 0 0 / 0.1)"
});

export function refreshCanvasColors() {
  if (typeof window === "undefined") return;
  const cs = getComputedStyle(document.documentElement);
  const read = (prop: string, fallback: string) => {
    const val = cs.getPropertyValue(prop).trim();
    return val || fallback;
  };

  canvasColors.bg = read("--canvas-bg", "oklch(0.29 0 0)");
  canvasColors.title = read("--canvas-text-title", "oklch(1 0 0)");
  canvasColors.artist = read("--canvas-text-artist", "oklch(1 0 0 / 0.7)");
  canvasColors.shadowOuter = read("--canvas-shadow-outer", "oklch(0 0 0 / 0.3)");
  canvasColors.shadowInner = read("--canvas-shadow-inner", "oklch(0 0 0 / 0.1)");
}

let observer: MutationObserver | null = null;

export function initColors() {
  if (typeof window === "undefined") return;
  refreshCanvasColors();
  if (!observer) {
    observer = new MutationObserver(() => {
      refreshCanvasColors();
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["style", "class"]
    });
  }
}
