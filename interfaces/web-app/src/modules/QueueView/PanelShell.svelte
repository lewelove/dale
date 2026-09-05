<script lang="ts">
import type { Snippet } from "svelte";

let {
  side = "left",
  children
}: {
  side?: "left" | "right";
  children?: Snippet;
} = $props();
</script>

<div class="panel-shell" class:left={side === "left"} class:right={side === "right"}>
  <div class="panel-inner">
    {@render children?.()}
  </div>
</div>

<style>
.panel-shell {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: none;
}

.panel-shell.left {
  clip-path: inset(0 -30px 0 0);
}

.panel-shell.right {
  clip-path: inset(0 0 0 -30px);
}

.panel-inner {
  flex: 1;
  padding: 20px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  text-shadow: var(--panel-text-shadow);
  transition: text-shadow 0.3s ease;
  min-height: 0;
}
</style>
