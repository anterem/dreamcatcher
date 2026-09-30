import { commands, events, type Section, type Snapshot } from './bindings';

function unwrap<T>(section: Section<T[]> | undefined) {
  return {
    loading: section === undefined,
    error: section?.status === 'error' ? section.error : '',
    data: section?.status === 'ok' ? section.data : []
  };
}

function createSave() {
  let current = $state<Snapshot | null>(null);
  let listening = false;

  async function refresh() {
    const res = await commands.getSnapshot();
    if (res.status === 'ok') current = res.data;
  }

  async function init() {
    if (listening) return;
    await events.saveChanged.listen((e) => {
      current = e.payload;
    });
    listening = true;
    await refresh();
  }

  return {
    get current() {
      return current;
    },
    get critters() {
      return unwrap(current?.critters);
    },
    get villagers() {
      return unwrap(current?.villagers);
    },
    init,
    refresh
  };
}

export const save = createSave();
