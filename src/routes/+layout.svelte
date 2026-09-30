<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { save } from '$lib/save.svelte';
  import { clock } from '$lib/clock.svelte';
  import { formatRelativeTime } from '$lib/utils';
  import '../app.css';

  let { children } = $props();

  let pathname = $derived(page.url.pathname);
  let isSelect = $derived(pathname === '/select');
  let lastUpdated = $derived(
    save.current ? formatRelativeTime(save.current.modifiedSecs, clock.nowSecs * 1000) : null
  );

  let navItems = $derived([
    { href: '/', label: 'Checklist', icon: '✅' },
    { href: '/critters', label: 'Critters', icon: '🐿️' },
    { href: '/villagers', label: 'Villagers', icon: '🎁' },
    { href: '/settings', label: 'Settings', icon: '⚙️' }
  ]);

  onMount(() => {
    save.init();
  });
</script>

{#if isSelect}
  {@render children()}
{:else}
  <div class="wrapper">
    <aside class="sidebar">
      <div class="logo">
        <img src="/star.svg" alt="" width="26" height="26" />
        <div class="logo-text">
          <span class="title">Dreamcatcher</span>
        </div>
      </div>
      <nav aria-label="Sections">
        {#each navItems as item (item.href)}
          <a
            class="nav-button"
            class:active={pathname === item.href}
            href={item.href}
            title={item.label}
          >
            <span class="icon" aria-hidden="true">{item.icon}</span>
            <span class="label">{item.label}</span>
          </a>
        {/each}
      </nav>
    </aside>
    <div class="content">
      {@render children()}
      {#if lastUpdated}
        <p class="save-time subtext dim">Saved {lastUpdated}</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .wrapper {
    max-width: 1280px;
    margin: 0 auto;
    min-height: 100dvh;
    display: flex;
  }

  .content {
    flex: 1;
    min-width: 0;
  }

  .sidebar {
    position: sticky;
    top: 0;
    height: 100dvh;
    width: 56px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px;
    border-right: 1px solid var(--separator);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px 14px;
  }

  .logo img {
    flex: none;
  }

  .logo-text {
    display: none;
    min-width: 0;
  }

  .title {
    font-size: 21px;
    font-weight: 700;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
  }

  .nav-button {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    width: 100%;
    height: 38px;
    border-radius: var(--radius);
    color: var(--subtext);
    font-size: 15px;
    font-weight: 500;
  }

  .nav-button:hover {
    background: var(--surface);
  }

  .nav-button.active {
    background: var(--accent);
    color: var(--text-contrast);
  }

  .icon {
    width: 20px;
    flex: none;
    text-align: center;
  }

  .label {
    display: none;
  }

  .save-time {
    padding: 0 28px 28px;
  }

  @media (min-width: 800px) {
    .sidebar {
      width: 218px;
      align-items: stretch;
    }

    .logo-text {
      display: block;
    }

    .nav-button {
      justify-content: flex-start;
      height: 32px;
      padding: 0 8px;
      text-align: left;
    }

    .label {
      display: block;
      flex: 1;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
  }
</style>
