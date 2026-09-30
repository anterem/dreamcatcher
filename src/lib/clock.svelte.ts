const TICK_MS = 30_000;

function createClock() {
  let nowSecs = $state(Math.floor(Date.now() / 1000));

  function sync() {
    nowSecs = Math.floor(Date.now() / 1000);
  }

  setInterval(sync, TICK_MS);
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) sync();
  });

  return {
    get nowSecs() {
      return nowSecs;
    }
  };
}

export const clock = createClock();
