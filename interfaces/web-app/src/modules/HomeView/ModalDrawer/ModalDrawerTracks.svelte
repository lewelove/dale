<script lang="ts">
import type { TrackItem } from "../../../types.ts";

let {
  tracks = [],
  totalDiscs = 1,
  albumArtist = "",
  onplay,
  onplaydisc
}: {
  tracks?: TrackItem[];
  totalDiscs?: string | number;
  albumArtist?: string;
  onplay?: (index: number) => void;
  onplaydisc?: (disc: number) => void;
} = $props();

let multiDisc = $derived(Number(totalDiscs) > 1);

function formatDuration(str?: string) {
  if (!str) {
    return "0:00";
  }

  const parts = str.split(":");

  while (parts.length > 2 && parseInt(parts[0]) === 0) {
    parts.shift();
  }

  if (parts[0].length > 1 && parts[0].startsWith("0")) {
    parts[0] = parts[0].substring(1);
  }

  return parts.join(":");
}

function formatMs(ms: number) {
  if (!ms) {
    return "0:00";
  }
  const totalSeconds = Math.floor(ms / 1000);
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;

  const pad = (num: number) => String(num).padStart(2, "0");

  if (h > 0) {
    return `${h}:${pad(m)}:${pad(s)}`;
  }
  return `${m}:${pad(s)}`;
}

function getDiscDuration(discNumber: number) {
  const totalMs = tracks
    .filter((t) => t.discnumber === discNumber)
    .reduce((acc, t) => acc + (Number(t.info?.duration_milliseconds) || 0), 0);
  return formatMs(totalMs);
}

function handlePlay(index: number) {
  if (onplay) {
    onplay(index);
  }
}

function handlePlayDisc(discNumber: number) {
  if (onplaydisc) {
    onplaydisc(discNumber);
  } else {
    const firstIndex = tracks.findIndex((t) => t.discnumber === discNumber);
    if (firstIndex !== -1 && onplay) {
      onplay(firstIndex);
    }
  }
}
</script>

<div class="tracks-list">
  {#each tracks as track, i (track.id || `${track.discnumber}-${track.tracknumber}-${i}`)}
    {#if multiDisc && (i === 0 || track.discnumber !== tracks[i - 1].discnumber)}
      {#if i > 0}
        <div class="disc-separator"></div>
      {/if}
      <div class="disc-header-row">
        <span class="disc-label">Disc {track.discnumber}</span>

        <div class="disc-header-right">
          <span class="v-mono disc-duration-label">{getDiscDuration(track.discnumber)}</span>
          <button
            type="button"
            class="v-btn-icon disc-play-btn"
            onclick={() => handlePlayDisc(track.discnumber)}
            title="Play Disc {track.discnumber}"
          >
            <span class="icon filled disc-play-icon">play_arrow</span>
          </button>
        </div>
      </div>
    {/if}

    <div
      class="v-track-row track-row"
      ondblclick={() => handlePlay(i)}
      onkeydown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          handlePlay(i);
        }
      }}
      role="button"
      tabindex="0"
      aria-label="Track {track.tracknumber}: {track.title}"
    >
      <span class="v-mono v-track-index track-index">{track.tracknumber}</span>
      <div class="v-track-body track-body">
        <span class="v-truncate v-track-title track-title">{track.title}</span>
        {#if track.artist && albumArtist && track.artist.toLowerCase() !== albumArtist.toLowerCase()}
          <span class="v-truncate v-track-artist track-artist">{track.artist}</span>
        {/if}
      </div>
      <span class="v-mono v-track-meta track-meta"
        >{formatDuration(track.info?.duration_formatted)}</span
      >
    </div>
  {/each}
</div>

<style>
.tracks-list {
  display: flex;
  flex-direction: column;
  width: 100%;
  background-color: transparent;
  box-sizing: border-box;
}

.track-row + .track-row {
  margin-top: 4px;
}

.disc-separator {
  height: 1px;
  background-color: var(--border-muted);
  margin: 12px 0;
  width: 100%;
}

.disc-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  user-select: none;
  margin-bottom: 8px;
}

.disc-header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.disc-label,
.disc-duration-label {
  display: flex;
  align-items: center;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-subtle);
  background-color: transparent;
  border: 1px solid var(--border-muted);
  border-radius: 8px;
  height: 24px;
  box-sizing: border-box;
}

.disc-duration-label {
  font-weight: 400;
}

.disc-play-btn {
  width: 32px;
  height: 24px;
  border-radius: 18px;
}

.disc-play-icon {
  font-size: 14px;
}

.track-index {
  color: var(--text-subtle);
}

.track-title {
  word-break: keep-all;
  overflow-wrap: break-word;
  color: var(--text-main);
}

.track-artist {
  color: var(--text-muted);
}

.track-meta {
  color: var(--text-subtle);
}
</style>
