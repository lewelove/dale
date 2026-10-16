import type { MpdStatusPayload, QueueItem } from "../types.ts";

export const player = $state<{
  state: string;
  currentAlbumId: string | null;
  currentTrackIndex: number | null;
  currentFile: string | null;
  title: string;
  artist: string;
  elapsed: number;
  duration: number;
  lastUpdated: number;
  queue: QueueItem[];
}>({
  state: "stop",
  currentAlbumId: null,
  currentTrackIndex: null,
  currentFile: null,
  title: "",
  artist: "",
  elapsed: 0,
  duration: 0,
  lastUpdated: 0,
  queue: []
});

export function updatePlayerState(data: MpdStatusPayload) {
  player.state = data.state;
  player.currentAlbumId = data.album_id ?? null;
  player.currentTrackIndex = data.track_index ?? null;
  player.currentFile = data.file ?? null;
  player.title = data.title || "";
  player.artist = data.artist || "";
  player.elapsed =
    typeof data.elapsed === "number" ? data.elapsed : parseFloat(String(data.elapsed || 0));
  player.duration =
    typeof data.duration === "number" ? data.duration : parseFloat(String(data.duration || 0));
  player.lastUpdated = performance.now();

  if (data.queue) {
    player.queue = data.queue;
  }
}
