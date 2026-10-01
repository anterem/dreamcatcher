<script lang="ts">
  import { save } from '$lib/save.svelte';
  import { checklistItems } from '$lib/checklist';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let section = $derived(save.current?.checklist ?? null);
  let items = $derived(section?.status === 'ok' ? checklistItems(section.data) : null);
  let starPaths = $derived(section?.status === 'ok' ? section.data.starPaths : []);
</script>

<section class="page">
  <PageHeader title="Checklist" />
  {#if section?.status === 'error'}
    <p class="message">Error: {section.error}</p>
  {:else if items === null}
    <p class="message"><em>Reading your save…</em></p>
  {:else if items.length === 0 && starPaths.length === 0}
    <p class="message">All done! No checklist items remaining.</p>
  {:else}
    <div class="grid">
      {#each items as item}
        <div class="item">
          <span class="name">{item.label}</span>
          {#if item.description}<span class="subtext">{item.description}</span>{/if}
        </div>
      {/each}
      {#each starPaths as group (group.name)}
        <div class="group-heading">{group.name}</div>
        {#each group.duties as duty}
          <div class="item">
            <span class="name">
              {duty.description}
              {#if duty.target > 1}<span class="chip">{duty.progress}/{duty.target}</span>{/if}
            </span>
          </div>
        {/each}
      {/each}
    </div>
  {/if}
</section>
