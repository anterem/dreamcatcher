<script lang="ts">
  import type { Role } from '$lib/bindings';
  import { save } from '$lib/save.svelte';
  import { clock } from '$lib/clock.svelte';
  import { villagerStatus, type VillagerStatus } from './status';
  import { ROLES, ROLE_ICONS, roleLabel } from '$lib/utils';
  import { PersistedState } from '$lib/persisted.svelte';
  import FilterToggle from '$lib/components/FilterToggle.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  const onlyGiftable = new PersistedState('villagers.toGift', false);
  const selectedRole = new PersistedState<Role | null>('villagers.role', null);

  let allVillagers = $derived(
    save.villagers.data.map((v) =>
      villagerStatus(v, save.current?.modifiedSecs ?? 0, clock.nowSecs, save.current?.tzOffset ?? 0)
    )
  );

  let toGiftCount = $derived(allVillagers.filter((v) => v.giftable).length);

  let shownVillagers = $derived.by(() => {
    let list = allVillagers;
    if (onlyGiftable.current) list = list.filter((v) => v.giftable);
    if (selectedRole.current !== null) list = list.filter((v) => v.role === selectedRole.current);
    return list;
  });

  let roleCounts = $derived.by(() => {
    const counts = new Map<Role, number>();
    for (const v of allVillagers) {
      if (v.role !== null) {
        counts.set(v.role, (counts.get(v.role) ?? 0) + 1);
      }
    }
    return counts;
  });

  let toLevel = $derived(shownVillagers.filter((v) => !v.isMaxed));
  let bestFriends = $derived(shownVillagers.filter((v) => v.isMaxed));

  function selectRole(role: Role) {
    selectedRole.current = selectedRole.current === role ? null : role;
  }
</script>

<section class="page">
  <PageHeader title="Villagers" />

  <div class="toolbar">
    <FilterToggle
      label={`To gift (${toGiftCount})`}
      active={onlyGiftable.current}
      onclick={() => (onlyGiftable.current = !onlyGiftable.current)}
    />
    {#each ROLES as role (role)}
      <FilterToggle
        label={`${ROLE_ICONS[role]} ${roleLabel(role)} (${roleCounts.get(role) ?? 0})`}
        active={selectedRole.current === role}
        onclick={() => selectRole(role)}
      />
    {/each}
  </div>

  {#if save.villagers.loading}
    <p class="message"><em>Reading your save…</em></p>
  {:else if save.villagers.error}
    <p class="message">Error: {save.villagers.error}</p>
  {:else if save.villagers.data.length === 0}
    <p class="message">No villager data found.</p>
  {:else}
    <div class="grid wide">
      {#snippet card(villager: VillagerStatus)}
        <div class="item villager">
          <span class="name">
            {villager.name}
            {#if villager.role}
              <span class="role" title={roleLabel(villager.role)}>{ROLE_ICONS[villager.role]}</span>
            {/if}
            {#if !villager.isMaxed}
              <span class="level">({villager.friendshipLevel})</span>
            {/if}
          </span>
          {#if villager.giftsAreCurrent}
            <span class="chips">
              {#each villager.gifts as gift (gift.itemId)}
                <span class="chip" class:done={gift.gifted} class:dim={gift.gifted}
                  >{gift.name}</span
                >
              {/each}
            </span>
          {/if}
        </div>
      {/snippet}
      <div class="group-heading">To level</div>
      {#each toLevel as villager (villager.id)}
        {@render card(villager)}
      {:else}
        <p class="empty">Nothing to level.</p>
      {/each}
      <div class="group-heading">Best friends</div>
      {#each bestFriends as villager (villager.id)}
        {@render card(villager)}
      {:else}
        <p class="empty">None yet.</p>
      {/each}
    </div>
  {/if}
</section>

<style>
  .role {
    font-size: 15px;
  }

  .level {
    font-size: 17px;
    font-weight: 600;
    color: var(--text);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .empty {
    padding: 7px 8px;
    font-size: 15px;
    color: var(--subtext);
  }
</style>
