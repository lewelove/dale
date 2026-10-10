import appConfig from "../config.json";

export const config = $state(structuredClone(appConfig));

export function updateConfig(newData: any) {
  if (!newData) return;

  if (newData.shader) {
    Object.assign(config.shader, newData.shader);
  }

  if (newData.album_grid) {
    if (newData.album_grid.spacing) {
      Object.assign(config.album_grid.spacing, newData.album_grid.spacing);
    }

    if (newData.album_grid.album_card) {
      if (newData.album_grid.album_card.cover) {
        Object.assign(config.album_grid.album_card.cover, newData.album_grid.album_card.cover);
      }

      if (newData.album_grid.album_card.text) {
        if (newData.album_grid.album_card.text.title) {
          Object.assign(
            config.album_grid.album_card.text.title,
            newData.album_grid.album_card.text.title
          );
        }
        if (newData.album_grid.album_card.text.albumartist) {
          Object.assign(
            config.album_grid.album_card.text.albumartist,
            newData.album_grid.album_card.text.albumartist
          );
        }
        if (newData.album_grid.album_card.text.spacing) {
          Object.assign(
            config.album_grid.album_card.text.spacing,
            newData.album_grid.album_card.text.spacing
          );
        }
        if (newData.album_grid.album_card.text.enable !== undefined) {
          config.album_grid.album_card.text.enable = newData.album_grid.album_card.text.enable;
        }
      }
    }
  }
}

if (import.meta.hot) {
  import.meta.hot.accept("../config.json", (mod: any) => {
    if (mod) {
      updateConfig(mod.default ?? mod);
    }
  });
}
