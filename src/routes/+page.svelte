<script lang="ts">
  import { save } from '$lib/save.svelte';
  import { checklistDescription, checklistLabel, liveChecklist } from '$lib/checklist';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let section = $derived(save.current?.checklist ?? null);
  let items = $derived(section?.status === 'ok' ? liveChecklist(section.data) : null);
</script>

<section class="page">
  <PageHeader title="Checklist" />
  {#if section?.status === 'error'}
    <p class="message">Error: {section.error}</p>
  {:else if items === null}
    <p class="message"><em>Reading your save…</em></p>
  {:else if items.length === 0}
    <p class="message">All done! No checklist items remaining.</p>
  {:else}
    <div class="grid">
      {#each items as item}
        {@const description = checklistDescription(item)}
        <div class="item">
          <span class="name">{checklistLabel(item)}</span>
          {#if description}<span class="subtext">{description}</span>{/if}
        </div>
      {/each}
    </div>
  {/if}
</section>
