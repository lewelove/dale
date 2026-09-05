<script lang="ts">
import { player } from "../player.svelte.ts";
import { collection } from "../../library/collection.svelte.ts";
import { view } from "../../library/view.svelte.ts";
import { nav } from "../../navigation.svelte.ts";
import { deriveColorTokens } from "../../colors.svelte.ts";

import BackgroundShader from "./BackgroundShader.svelte";
import NavigationBar from "../NavigationBar.svelte";
import CoverPanel from "./CoverPanel.svelte";
import TrackPanel from "./TrackPanel.svelte";
import QueuePanel from "./QueuePanel.svelte";
import Sidebar from "./Sidebar.svelte";

let activeId = $derived(player.currentAlbumId);
let activeAlbum = $derived(activeId ? collection.dict[activeId] : null);
let coverHash = $derived(activeAlbum?.cover_hash || "");
let fullAlbum = $derived(activeId ? collection.fullAlbumCache[activeId] : null);

let palette = $derived(
  fullAlbum?.album?.colors?.background || activeAlbum?.colors?.background || []
);
let hasPalette = $derived(palette && palette.length > 0);
let foregroundColor = $derived(
  fullAlbum?.album?.colors?.foreground || activeAlbum?.colors?.foreground || null
);

let isShaderOn = $derived(view.isShaderActive && hasPalette);
let activeForeground = $derived(isShaderOn && foregroundColor ? foregroundColor : null);

let queueContainerStyle = $derived.by(() => {
  if (!activeForeground) return "";
  const tokens = deriveColorTokens(activeForeground);
  return Object.entries(tokens)
    .map(([k, v]) => `${k}: ${v};`)
    .join(" ");
});

let isViewVisible = $derived(nav.activeTab === "queue");
let isPlaying = $derived(player.state === "play");

let moduleWidth = $state(0);

$effect(() => {
  const uniqueIds = [...new Set(player.queue.map((item) => item.album_id).filter(Boolean))];
  if (activeId && !uniqueIds.includes(activeId)) {
    uniqueIds.push(activeId);
  }
  uniqueIds.forEach((id) => collection.ensureFullAlbum(id));
});
</script>

<div
  class="queue-view-container"
  class:shader-off={!isShaderOn}
  style={queueContainerStyle}
>
  <BackgroundShader colors={palette} coverSize={moduleWidth} visible={isViewVisible} {isPlaying} />

  <div class="queue-layout">
    <div class="left-wing">
      <NavigationBar variant="transparent" />
      <TrackPanel />
    </div>

    <div class="center-wing" bind:clientWidth={moduleWidth}>
      <CoverPanel {coverHash} width={moduleWidth} />
    </div>

    <div class="right-wing">
      <QueuePanel />
      <Sidebar {hasPalette} />
    </div>
  </div>
</div>

<style>
.queue-view-container {
  width: 100%;
  height: 100%;
  background-color: var(--bg-queue);
  position: relative;
  overflow: hidden;
}

.queue-layout {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: row;
  z-index: 1;
  position: relative;
}

.left-wing {
  flex: 1 1 0%;
  display: flex;
  min-width: 300px;
  height: 100%;
  flex-direction: row;
}

.right-wing {
  flex: 1 1 0%;
  display: flex;
  min-width: 300px;
  height: 100%;
  flex-direction: row;
}

.center-wing {
  flex: 0 1 auto;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  box-sizing: border-box;
  min-width: 0;
}
</style>
