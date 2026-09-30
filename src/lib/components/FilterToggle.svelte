<script lang="ts">
  let {
    label,
    active = false,
    onclick
  }: {
    label: string;
    active?: boolean;
    onclick: () => void;
  } = $props();
</script>

<button type="button" class="toggle" class:active aria-pressed={active} {onclick}>
  {label}
</button>

<style>
  .toggle {
    height: 26px;
    padding: 0 9px;
    border-radius: var(--radius);
    color: var(--subtext);
    font-size: 15px;
    font-weight: 500;
    white-space: nowrap;
  }

  .toggle:hover {
    color: var(--text);
    background: var(--surface);
  }

  .toggle.active {
    background: var(--overlay);
    color: var(--text);
  }
</style>
