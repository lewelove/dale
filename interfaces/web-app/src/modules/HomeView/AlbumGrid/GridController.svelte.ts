import { LayoutManager } from "./Layout.svelte.ts";
import { ScrollEngine } from "./Scroll.svelte.ts";
import type { AlbumCardItem } from "../../../types.ts";

export class GridController {
  layout = new LayoutManager();
  scroll = new ScrollEngine();
  viewportHeight: number = $state(0);

  albums: AlbumCardItem[] = $state([]);

  allRows = $derived(this.layout.chunk(this.albums));

  visibleRows = $derived(Math.ceil(this.viewportHeight / this.layout.rowHeight));

  maxSlots = $derived(Math.max(0, this.allRows.length + 1 - this.visibleRows));

  virtualRows = $derived.by(() => {
    if (this.allRows.length === 0) {
      return [];
    }

    const { start, end } = this.layout.getVisibleIndices(
      this.scroll.currentY,
      this.viewportHeight,
      this.allRows.length
    );

    const indicesToRender: number[] = [];
    for (let i = start; i <= end; i++) {
      indicesToRender.push(i);
    }

    return indicesToRender.map((i) => ({
      index: i,
      y: this.layout.getRowY(i),
      data: this.allRows[i]
    }));
  });

  update(dpr: number = 1) {
    this.scroll.update(this.layout.rowHeight, dpr);
  }

  handleWheel(e: WheelEvent) {
    this.scroll.handleWheel(e, this.maxSlots);
  }

  scrollRow(delta: number) {
    const newSlot = this.scroll.targetSlot + delta;
    this.scroll.targetSlot = Math.max(0, Math.min(newSlot, this.maxSlots));
  }

  resetScroll() {
    this.scroll.syncToSlot(0);
    this.scroll.currentY = 0;
  }
}
