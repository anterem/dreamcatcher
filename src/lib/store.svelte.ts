import type { SaveFile } from './bindings';

function createLoadedSaveFile() {
  let current = $state<SaveFile | null>(null);

  return {
    get current(): SaveFile | null {
      return current;
    },
    set current(file: SaveFile | null) {
      current = file;
    }
  };
}

export const loadedSaveFile = createLoadedSaveFile();
