import appConfig from "../config.json";

export type AppConfig = typeof appConfig;

export const config = $state<AppConfig>(structuredClone(appConfig));

export function updateConfig(newData: Record<string, unknown> | null | undefined) {
  if (!newData) {
    return;
  }

  const shader = newData.shader as Partial<AppConfig["shader"]> | undefined;
  if (shader) {
    Object.assign(config.shader, shader);
  }

  const grid = newData.album_grid as Partial<AppConfig["album_grid"]> | undefined;
  if (grid) {
    if (grid.spacing) {
      Object.assign(config.album_grid.spacing, grid.spacing);
    }

    if (grid.album_card) {
      if (grid.album_card.cover) {
        Object.assign(config.album_grid.album_card.cover, grid.album_card.cover);
      }

      if (grid.album_card.text) {
        if (grid.album_card.text.title) {
          Object.assign(
            config.album_grid.album_card.text.title,
            grid.album_card.text.title
          );
        }
        if (grid.album_card.text.albumartist) {
          Object.assign(
            config.album_grid.album_card.text.albumartist,
            grid.album_card.text.albumartist
          );
        }
        if (grid.album_card.text.spacing) {
          Object.assign(
            config.album_grid.album_card.text.spacing,
            grid.album_card.text.spacing
          );
        }
        if (grid.album_card.text.enable !== undefined) {
          config.album_grid.album_card.text.enable = grid.album_card.text.enable;
        }
      }
    }
  }
}

if (import.meta.hot) {
  import.meta.hot.accept("../config.json", (mod) => {
    if (mod) {
      const nextConfig = ("default" in mod ? mod.default : mod) as Record<string, unknown>;
      updateConfig(nextConfig);
    }
  });
}
