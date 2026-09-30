<script lang="ts">
  import { save } from '$lib/save.svelte';
  import { clock } from '$lib/clock.svelte';
  import { localHour, localWeekday, upcomingWindowStart } from '$lib/time';
  import { critterStatus, type CritterStatus } from './status';
  import { WEEKDAY_NAMES, formatHour, formatSchedule } from '$lib/utils';
  import { PersistedState } from '$lib/persisted.svelte';
  import FilterToggle from '$lib/components/FilterToggle.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let tz = $derived(save.current?.tzOffset ?? 0);
  let todayIndex = $derived(localWeekday(clock.nowSecs, tz));
  let selectedDay = $state<number | null>(null);
  let activeDay = $derived(selectedDay ?? todayIndex);
  let isToday = $derived(activeDay === todayIndex);
  let prevDayName = $derived(WEEKDAY_NAMES[(activeDay + 6) % 7]);
  let nextDayName = $derived(WEEKDAY_NAMES[(activeDay + 1) % 7]);
  let nowHour = $derived(localHour(clock.nowSecs, tz));
  let allCritters = $derived(save.critters.data.map((c) => critterStatus(c, clock.nowSecs, tz)));

  const onlyToFeed = new PersistedState('critters.toFeed', false);
  const onlyUntamed = new PersistedState('critters.untamed', false);

  let dayCritters = $derived.by(() => {
    let list = allCritters.filter((c) => c.schedule[activeDay].length > 0);
    if (isToday && onlyToFeed.current) list = list.filter((c) => c.needsFeeding);
    if (onlyUntamed.current) list = list.filter((c) => !c.tamed);
    return list.toSorted(compareOnDay(activeDay));
  });
  let available = $derived(isToday ? dayCritters.filter((c) => c.availableNow) : dayCritters);
  let unavailable = $derived(isToday ? dayCritters.filter((c) => !c.availableNow) : []);

  function prevDay() {
    selectedDay = (activeDay + 6) % 7;
  }

  function nextDay() {
    selectedDay = (activeDay + 1) % 7;
  }

  function goToToday() {
    selectedDay = null;
  }

  function compareOnDay(day: number) {
    return (a: CritterStatus, b: CritterStatus) => {
      const sa = a.schedule[day][0];
      const sb = b.schedule[day][0];
      return sa.start - sb.start || sb.end - sa.end || a.speciesRank - b.speciesRank;
    };
  }

  function availableAt(critter: CritterStatus): string | null {
    if (!isToday || critter.availableNow || critter.fedToday) return null;
    const opens = upcomingWindowStart(critter.schedule[activeDay], nowHour);
    return opens === null ? null : `opens at ${formatHour(opens)}`;
  }
</script>

<section class="page">
  <PageHeader title="Critters">
    <span class="day-switch">
      <button class="day-arrow" onclick={prevDay} aria-label={`Previous day, ${prevDayName}`}
        >‹</button
      >
      <button
        class="day-label"
        onclick={goToToday}
        disabled={isToday}
        title={isToday ? undefined : 'Return to today'}
      >
        {WEEKDAY_NAMES[activeDay]}{#if isToday}<span>(Today)</span>{/if}
      </button>
      <button class="day-arrow" onclick={nextDay} aria-label={`Next day, ${nextDayName}`}>›</button>
    </span>
  </PageHeader>

  <div class="toolbar">
    <span class="toggle-group">
      <FilterToggle
        label="To feed"
        active={onlyToFeed.current}
        onclick={() => (onlyToFeed.current = !onlyToFeed.current)}
      />
      <FilterToggle
        label="Untamed"
        active={onlyUntamed.current}
        onclick={() => (onlyUntamed.current = !onlyUntamed.current)}
      />
    </span>
  </div>

  {#if save.critters.loading}
    <p class="message"><em>Reading your save…</em></p>
  {:else if save.critters.error}
    <p class="message">Error: {save.critters.error}</p>
  {:else if save.critters.data.length === 0}
    <p class="message">No critter data found.</p>
  {:else if dayCritters.length === 0}
    <p class="message">No critters on the schedule.</p>
  {:else}
    <div class="grid wide">
      {#snippet card(critter: CritterStatus, dim = false)}
        {@const nextAvailableAt = availableAt(critter)}
        <div class="item critter" class:dim>
          <span class="name">
            {critter.name}
            {#if isToday && critter.fedToday}<span class="chip accent">✓</span>{/if}
            {#if critter.tamed}<span class="chip tamed">♥</span>{/if}
          </span>
          <span class="subtext">
            {critter.biome} · {formatSchedule(critter.schedule[activeDay])}
            {#if critter.notes.length > 0}
              · {critter.notes.join(', ')}{/if}
            {#if nextAvailableAt}
              <span class="accent"> · {nextAvailableAt}</span>
            {/if}
          </span>
        </div>
      {/snippet}
      {#if isToday}
        <div class="group-heading">Available now</div>
        {#each available as critter (critter.itemId)}
          {@render card(critter)}
        {:else}
          <p class="empty">No critters.</p>
        {/each}
        <div class="group-heading">Unavailable</div>
        {#each unavailable as critter (critter.itemId)}
          {@render card(critter, true)}
        {:else}
          <p class="empty">No critters.</p>
        {/each}
      {:else}
        {#each dayCritters as critter (critter.itemId)}{@render card(critter)}{/each}
      {/if}
    </div>
  {/if}
</section>

<style>
  .day-switch {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }

  .day-label {
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    font-size: 15px;
    font-weight: 700;
    white-space: nowrap;
  }

  .day-label:hover:not(:disabled) {
    color: var(--text);
  }

  .day-label:disabled {
    cursor: default;
  }

  .day-label span {
    color: var(--subtext);
    font-weight: 400;
    font-size: 13px;
  }

  .day-arrow {
    width: 26px;
    height: 26px;
    border-radius: var(--radius);
    color: var(--subtext);
    font-size: 14px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .day-arrow:hover {
    background: var(--surface);
    color: var(--text);
  }

  .accent {
    color: var(--accent);
  }

  .tamed {
    color: var(--soft);
  }

  .empty {
    padding: 7px 8px;
    font-size: 15px;
    color: var(--subtext);
  }
</style>
