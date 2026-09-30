<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { commands, type SaveFile } from '$lib/bindings';
  import { loadedSaveFile } from '$lib/store.svelte';
  import { save } from '$lib/save.svelte';
  import { formatRelativeTime } from '$lib/utils';
  import { clock } from '$lib/clock.svelte';

  let saveFiles: SaveFile[] = $state([]);
  let saveFilesError = $state('');
  let loadError = $state('');
  let loading = $state(false);

  async function getSaveFiles() {
    const res = await commands.getSaveFiles();
    if (res.status !== 'ok') {
      saveFilesError = res.error;
      return;
    }
    saveFiles = res.data;
    if (saveFiles.length === 1) load(saveFiles[0]);
  }

  async function load(file: SaveFile) {
    loading = true;
    loadError = '';
    const result = await commands.loadSaveFile(file.path, file.storefront);
    if (result.status === 'ok') {
      await save.refresh();
      loadedSaveFile.current = file;
      goto('/');
    } else {
      loading = false;
      loadError = result.error;
    }
  }

  onMount(getSaveFiles);
</script>

<main>
  <p class="title">Dreamcatcher</p>
  {#if saveFilesError}
    <p class="message">Error: {saveFilesError}</p>
  {:else if loadError}
    <p class="message">Could not read save: {loadError}</p>
  {:else if loading}
    <p class="message"><em>Reading save file…</em></p>
  {:else if saveFiles.length === 0}
    <p class="message">No save files were found in the usual places.</p>
  {:else}
    <p class="message"><em>Choose a save to continue.</em></p>
    <ul class="saves">
      {#each saveFiles as file (file.path)}
        <li>
          <button onclick={() => load(file)}>
            <span class="storefront">{file.storefront}</span>
            <span class="when"
              >saved {formatRelativeTime(file.lastModified, clock.nowSecs * 1000)}</span
            >
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  main {
    height: 100dvh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-5);
  }

  .title {
    font-size: 28px;
    font-weight: 700;
    letter-spacing: -0.3px;
    margin-bottom: var(--space-6);
  }

  .saves {
    list-style: none;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: min(100%, 320px);
  }

  .saves button {
    width: 100%;
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-3);
    padding: 10px 14px;
    border-radius: var(--radius);
    background: var(--surface);
    text-align: left;
  }

  .saves button:hover {
    background: var(--overlay);
  }

  .storefront {
    font-size: 15px;
    font-weight: 600;
    text-transform: capitalize;
  }

  .when {
    font-size: 13px;
    color: var(--subtext);
  }
</style>
