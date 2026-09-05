<script lang="ts">
let {
  track = null,
  file = null
}: {
  track?: any;
  file?: string | null;
} = $props();

let isInstrumental = $derived(track?.keys?.instrumental === true);

let lyricsText = $derived.by(() => {
  if (!track || track.keys?.instrumental === true) {
    return "";
  }
  if (track.lyrics?.text) {
    return track.lyrics.text;
  }
  if (track.keys?.lyrics) {
    return track.keys.lyrics;
  }
  return "";
});
</script>

<div class="lyrics-scroll">
  {#key file}
    {#if isInstrumental}
      <div class="instrumental-msg">INSTRUMENTAL</div>
    {:else if lyricsText}
      <div class="lyrics-content">
        {#each lyricsText.split(/\r?\n/) as line, i (i)}
          <p class="lyric-line">{line}</p>
        {/each}
      </div>
    {/if}
  {/key}
</div>

<style>
.lyrics-scroll {
  flex: 1;
  overflow-y: auto;
  width: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.lyrics-scroll::-webkit-scrollbar {
  width: 0;
}

.lyrics-content {
  font-family: var(--font-stack);
  font-size: 15px;
  line-height: 1.05;
  color: var(--text-main);
  text-align: center;
  margin: -6px auto;
  width: 100%;
}

.lyric-line {
  margin: 6px 14px;
  min-height: 0.3em;
  text-wrap: balance;
}

.instrumental-msg {
  margin: auto;
  font-family: var(--font-mono);
  font-size: 15px;
  line-height: 1.2;
  color: var(--text-main);
  text-align: center;
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}
</style>
