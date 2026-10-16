<script lang="ts">
import { fade } from "svelte/transition";
import type { GrouperTreeNode } from "../../types.ts";

let {
  items = [],
  container = null
}: {
  items?: GrouperTreeNode[];
  container?: HTMLElement | null;
} = $props();

let indexContainer: HTMLDivElement | null = $state(null);
let charsWrapper: HTMLDivElement | null = $state(null);
let isScrubbing = $state(false);
let scrubChar = $state("");
let activeEntryIndex = $state(-1);
let bubbleY = $state(0);

function getBucketChar(label: string) {
  if (!label) {
    return "?";
  }
  const normalized = label
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toUpperCase();
  const char = normalized.charAt(0);
  if (/[A-Z]/.test(char)) {
    return char;
  }
  if (/[0-9]/.test(char)) {
    return "#";
  }
  return "?";
}

let indexEntries = $derived.by(() => {
  const entries: { char: string; itemIndex: number }[] = [];
  items.forEach((item, i) => {
    const rawKey = item.sort ?? item.label ?? "";
    const sortKey = String(rawKey);
    const char = getBucketChar(sortKey);
    if (entries.length === 0 || entries[entries.length - 1].char !== char) {
      entries.push({ char, itemIndex: i });
    }
  });
  return entries;
});

function calculateScrub(e: PointerEvent) {
  if (!charsWrapper || !indexContainer || indexEntries.length === 0) {
    return;
  }

  const wrapperRect = charsWrapper.getBoundingClientRect();
  const containerRect = indexContainer.getBoundingClientRect();

  const wrapperY = e.clientY - wrapperRect.top;
  bubbleY = Math.max(0, Math.min(e.clientY - containerRect.top, containerRect.height));

  let pct = wrapperY / wrapperRect.height;
  pct = Math.max(0, Math.min(1, pct));

  const entryIndex = Math.min(Math.floor(pct * indexEntries.length), indexEntries.length - 1);
  const targetEntry = indexEntries[entryIndex];

  if (activeEntryIndex !== entryIndex) {
    activeEntryIndex = entryIndex;
    scrubChar = targetEntry.char;

    if (container) {
      const targetEl = container.querySelector(`#sidebar-item-${targetEntry.itemIndex}`) as HTMLElement;
      if (targetEl) {
        container.scrollTop = targetEl.offsetTop - 12;
      }
    }
  }
}

function onPointerDown(e: PointerEvent) {
  if (e.button !== 0) {
    return;
  }
  isScrubbing = true;
  indexContainer?.setPointerCapture(e.pointerId);
  calculateScrub(e);
}

function onPointerMove(e: PointerEvent) {
  if (isScrubbing) {
    calculateScrub(e);
  }
}

function onPointerUp(e: PointerEvent) {
  isScrubbing = false;
  scrubChar = "";
  activeEntryIndex = -1;
  if (indexContainer && indexContainer.hasPointerCapture(e.pointerId)) {
    indexContainer.releasePointerCapture(e.pointerId);
  }
}
</script>

<div
  class="sidebar-index-container"
  bind:this={indexContainer}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  role="slider"
  aria-valuenow={0}
  tabindex="0"
>
  <div class="chars-wrapper" bind:this={charsWrapper}>
    {#each indexEntries as entry, idx (idx)}
      <div class="index-char" class:active={isScrubbing && activeEntryIndex === idx}>
        {entry.char}
      </div>
    {/each}
  </div>

  {#if isScrubbing}
    <div
      class="scrub-callout"
      style:top="{bubbleY}px"
      transition:fade={{ duration: 100 }}
    >
      {scrubChar}
    </div>
  {/if}
</div>

<style>
.sidebar-index-container {
  position: relative;
  width: 12px;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  touch-action: none;
  user-select: none;
  cursor: pointer;
  z-index: 50;
  margin-left: 12px;
  margin-right: 0;
  flex-shrink: 0;
}

.chars-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
  width: 100%;
}

.index-char {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-subtle);
  display: flex;
  align-items: center;
  justify-content: center;
  height: 12px;
  width: 100%;
}

.index-char.active {
  color: var(--text-main);
}

.scrub-callout {
  position: absolute;
  right: 24px;
  transform: translateY(-50%);
  width: 40px;
  height: 40px;
  border-radius: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 700;
  color: var(--text-main);
  pointer-events: none;
  box-shadow: var(--panel-shadow);
  background-color: var(--bg-panel);
  border: none;
}
</style>
