<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, children }: { title: string; children?: Snippet } = $props();
</script>

<header class="page-header">
  <h1>{title}</h1>
  {@render children?.()}
</header>

<style>
  .page-header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-wrap: wrap;
    padding: 14px 20px 10px;
    /* extend border to right edge */
    margin: 0 calc(-20px - max(0px, (100cqw - 1280px) / 2)) 12px -20px;
    border-bottom: 1px solid var(--separator);
    flex: none;
  }
</style>
