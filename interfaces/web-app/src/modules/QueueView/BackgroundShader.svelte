<script>
import { onMount, onDestroy } from "svelte";
import { config } from "../../config.svelte.ts";
import { view } from "../../library/view.svelte.ts";

import vertexShaderSource from "./Shaders/Quad.vert?raw";
import mountainEqShader from "../../../shaders/mountain_eq.frag?raw";
import mountainShader from "../../../shaders/mountain.frag?raw";
import simplexLegacyShader from "../../../shaders/simplex_legacy.frag?raw";
import simplexShader from "../../../shaders/simplex.frag?raw";

let { colors = [], coverSize = 0, visible = false, isPlaying = false } = $props();

const PALETTE_SIZE_LIMIT = 12;

const SHADERS = {
  mountain_eq: mountainEqShader,
  mountain: mountainShader,
  simplex_legacy: simplexLegacyShader,
  simplex: simplexShader
};

let canvasEl;
let gl;
let program;
let animationFrame;
let locations = {};

let totalTime = 0;
let lastFrameTime = 0;
let isTabVisible = $state(true);
let randomOffset = Math.random() * 1000.0;

const floatColorsOklab = new Float32Array(24 * 3);
const floatRatios = new Float32Array(24);
let activeColorCount = 0;

let shaderSource = $derived(
  SHADERS[config.shader?.name] || mountainEqShader
);

function parseHexToRgb(hex) {
  let clean = hex.replace("#", "").trim();
  if (clean.length === 3) {
    clean = clean.split("").map((c) => c + c).join("");
  }
  const num = parseInt(clean, 16);
  if (isNaN(num)) return [0.2, 0.2, 0.2];
  return [(num >> 16) & 255, (num >> 8) & 255, num & 255].map((v) => v / 255);
}

function srgbToLinear(c) {
  return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

function parseColorToOklab(colorStr) {
  if (!colorStr) return [0.26, 0, 0];
  if (Array.isArray(colorStr)) return [colorStr[0] || 0.26, colorStr[1] || 0, colorStr[2] || 0];
  const rgb = parseHexToRgb(colorStr);
  const r = srgbToLinear(rgb[0]);
  const g = srgbToLinear(rgb[1]);
  const b = srgbToLinear(rgb[2]);

  const l_ = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m_ = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s_ = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);

  const L = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
  const a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
  const bVal = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

  return [L, a, bVal];
}

function parseColorToOklch(colorStr) {
  const [L, a, b] = parseColorToOklab(colorStr);
  const C = Math.hypot(a, b);
  let H = (Math.atan2(b, a) * 180) / Math.PI;
  if (H < 0) H += 360;
  return { L, C, H };
}

function getChroma(c) {
  const val = Array.isArray(c) ? c[0] : c.hex || c;
  return parseColorToOklch(val).C;
}

function shuffle(array) {
  for (let i = array.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [array[i], array[j]] = [array[j], array[i]];
  }
  return array;
}

$effect(() => {
  let palette = colors && colors.length > 0 ? [...colors] : ["#3d3d3d"];
  const order = config.shader?.order || "original";

  if (order !== "original") {
    palette.sort((a, b) => getChroma(b) - getChroma(a));
  }

  palette = palette.slice(0, PALETTE_SIZE_LIMIT);

  if (order === "random") {
    shuffle(palette);
  } else if (order === "ratio") {
    palette.sort((a, b) => {
      const rA = Array.isArray(a) ? parseFloat(a[a.length - 1]) : 0;
      const rB = Array.isArray(b) ? parseFloat(b[b.length - 1]) : 0;
      return rB - rA;
    });
  } else if (order.startsWith("oklch,")) {
    const comp = order.split(",")[1];
    palette.sort((a, b) => {
      const valA = Array.isArray(a) ? a[0] : a.hex || a;
      const valB = Array.isArray(b) ? b[0] : b.hex || b;
      const oklchA = parseColorToOklch(valA);
      const oklchB = parseColorToOklch(valB);
      const cA = oklchA[comp] ?? 0;
      const cB = oklchB[comp] ?? 0;
      return cA - cB;
    });
  }

  activeColorCount = palette.length;

  let hasRatios = false;
  for (let i = 0; i < activeColorCount; i++) {
    if (Array.isArray(palette[i]) && palette[i].length > 1) {
      hasRatios = true;
      break;
    }
  }

  let rawRatios = new Array(activeColorCount).fill(0);
  let totalRaw = 0;

  for (let i = 0; i < activeColorCount; i++) {
    const c = palette[i];
    if (hasRatios) {
      rawRatios[i] = Array.isArray(c) ? parseFloat(c[c.length - 1]) : 0.0;
    } else {
      rawRatios[i] = 1.0 / (i + 1.0);
    }
    totalRaw += rawRatios[i];
  }

  if (totalRaw > 0) {
    for (let i = 0; i < activeColorCount; i++) {
      rawRatios[i] /= totalRaw;
    }
  } else {
    for (let i = 0; i < activeColorCount; i++) {
      rawRatios[i] = 1.0 / activeColorCount;
    }
  }

  const equalize = config.shader?.equalize ?? 0;
  const avgRatio = 1.0 / activeColorCount;

  for (let i = 0; i < activeColorCount; i++) {
    rawRatios[i] = rawRatios[i] * (1.0 - equalize) + avgRatio * equalize;
  }

  for (let i = 0; i < 24; i++) {
    if (i < activeColorCount) {
      const c = palette[i];
      const colorVal = Array.isArray(c) ? c[0] : c.hex || c;
      const [L, a, b] = parseColorToOklab(colorVal);

      floatColorsOklab[i * 3 + 0] = L;
      floatColorsOklab[i * 3 + 1] = a;
      floatColorsOklab[i * 3 + 2] = b;
      floatRatios[i] = rawRatios[i];
    } else {
      floatColorsOklab[i * 3 + 0] = 0.0;
      floatColorsOklab[i * 3 + 1] = 0.0;
      floatColorsOklab[i * 3 + 2] = 0.0;
      floatRatios[i] = 0.0;
    }
  }
});

function createShader(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    gl.deleteShader(shader);
    return null;
  }
  return shader;
}

function initGL() {
  if (!canvasEl) return;
  gl = canvasEl.getContext("webgl2", {
    alpha: false,
    antialias: true,
    premultipliedAlpha: false,
    preserveDrawingBuffer: true
  });

  if (!gl) return;

  const vs = createShader(gl, gl.VERTEX_SHADER, vertexShaderSource);
  const fs = createShader(gl, gl.FRAGMENT_SHADER, shaderSource);

  if (!vs || !fs) return;

  if (program) gl.deleteProgram(program);
  program = gl.createProgram();
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);

  locations = {
    iTime: gl.getUniformLocation(program, "iTime"),
    iRandom: gl.getUniformLocation(program, "iRandom"),
    iResolution: gl.getUniformLocation(program, "iResolution"),
    iCoverSize: gl.getUniformLocation(program, "iCoverSize"),
    iColorsOklab: gl.getUniformLocation(program, "iColorsOklab"),
    iRatios: gl.getUniformLocation(program, "iRatios"),
    iCount: gl.getUniformLocation(program, "iCount"),
    iSpeed: gl.getUniformLocation(program, "iSpeed"),
    iZoom: gl.getUniformLocation(program, "iZoom"),
    iBlur: gl.getUniformLocation(program, "iBlur"),
    iGrain: gl.getUniformLocation(program, "iGrain"),
    iEqualize: gl.getUniformLocation(program, "iEqualize")
  };

  const vertices = new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]);
  const buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.STATIC_DRAW);

  const positionLoc = gl.getAttribLocation(program, "position");
  gl.enableVertexAttribArray(positionLoc);
  gl.vertexAttribPointer(positionLoc, 2, gl.FLOAT, false, 0, 0);

  lastFrameTime = performance.now();
  startLoop();
}

$effect(() => {
  if (shaderSource) initGL();
});

function startLoop() {
  if (animationFrame) cancelAnimationFrame(animationFrame);
  lastFrameTime = performance.now();
  render();
}

function render() {
  if (!gl || !program) return;

  if (!visible || !isTabVisible || !view.isShaderActive) {
    animationFrame = requestAnimationFrame(render);
    return;
  }

  const now = performance.now();
  if (isPlaying) {
    let delta = (now - lastFrameTime) / 1000;
    if (delta > 0.1) delta = 0.016;
    totalTime += delta;
  }
  lastFrameTime = now;

  gl.viewport(0, 0, canvasEl.width, canvasEl.height);
  gl.useProgram(program);

  gl.uniform1f(locations.iTime, totalTime);
  gl.uniform1f(locations.iRandom, randomOffset);
  gl.uniform2f(locations.iResolution, canvasEl.width, canvasEl.height);

  const dpr = window.devicePixelRatio || 1;
  gl.uniform1f(locations.iCoverSize, coverSize * dpr);

  gl.uniform3fv(locations.iColorsOklab, floatColorsOklab);
  gl.uniform1fv(locations.iRatios, floatRatios);
  gl.uniform1i(locations.iCount, activeColorCount);

  const s = config.shader || {};
  gl.uniform1f(locations.iSpeed, s.speed ?? 0.007);
  gl.uniform1f(locations.iZoom, s.zoom ?? 0.4);
  gl.uniform1f(locations.iBlur, s.blur ?? 0.8);

  gl.uniform1f(locations.iGrain, s.grain ?? 0.01);
  gl.uniform1f(locations.iEqualize, s.equalize ?? 1.0);

  gl.drawArrays(gl.TRIANGLES, 0, 6);

  animationFrame = requestAnimationFrame(render);
}

function handleResize() {
  if (canvasEl) {
    const dpr = window.devicePixelRatio || 1;
    canvasEl.width = window.innerWidth * dpr;
    canvasEl.height = window.innerHeight * dpr;
  }
}

function handleVisibilityChange() {
  isTabVisible = !document.hidden;
}

$effect(() => {
  if (colors || coverSize || config.shader) {
    handleResize();
  }
});

$effect(() => {
  if (visible && isTabVisible) {
    lastFrameTime = performance.now();
  }
});

onMount(() => {
  handleResize();
  initGL();
  window.addEventListener("resize", handleResize);
  document.addEventListener("visibilitychange", handleVisibilityChange);
});

onDestroy(() => {
  if (animationFrame) cancelAnimationFrame(animationFrame);
  window.removeEventListener("resize", handleResize);
  document.removeEventListener("visibilitychange", handleVisibilityChange);
});
</script>

<canvas
  bind:this={canvasEl}
  style="
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    z-index: 0;
    pointer-events: none;
    transition: opacity 0.3s ease;
  "
  style:opacity={view.isShaderActive && colors.length > 0 ? 1 : 0}
></canvas>
