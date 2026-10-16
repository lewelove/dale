<script lang="ts">
import { view } from "../../library/view.svelte.ts";
import AlbumGrid from "./AlbumGrid/AlbumGrid.svelte";
import type { AlbumCardItem } from "../../types.ts";

let {
  activeAlbumId = null,
  onfocus = () => {}
}: {
  activeAlbumId?: string | null;
  onfocus?: (album: AlbumCardItem) => void;
} = $props();
</script>

<AlbumGrid
  albums={view.shelfAlbums}
  version={view.shelfVersion}
  resetVersion={view.shelfResetVersion}
  {activeAlbumId}
  {onfocus}
/>
