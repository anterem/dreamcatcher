import type { ChecklistFacts, ScroogeStore } from './bindings';

export type ChecklistItem =
  | { kind: 'moonstoneChest'; location: string | null }
  | { kind: 'rift'; location: string | null }
  | { kind: 'dreamSnapSubmission'; location: null }
  | { kind: 'dreamSnapVoting'; location: null }
  | ({ kind: 'scroogeStore' } & ScroogeStore);

export function liveChecklist(facts: ChecklistFacts): ChecklistItem[] {
  const items: ChecklistItem[] = facts.moonstoneChestBiomes.map((biome) => ({
    kind: 'moonstoneChest',
    location: biome
  }));
  for (const biome of facts.riftBiomes) items.push({ kind: 'rift', location: biome });
  if (facts.dreamSnaps?.submitNeeded) items.push({ kind: 'dreamSnapSubmission', location: null });
  if (facts.dreamSnaps?.voteNeeded) items.push({ kind: 'dreamSnapVoting', location: null });
  for (const store of facts.scroogeStores) items.push({ kind: 'scroogeStore', ...store });
  return items;
}

export function checklistLabel(item: ChecklistItem): string {
  switch (item.kind) {
    case 'moonstoneChest':
      return 'Daily Moonstone Chest';
    case 'rift':
      return 'Time Rift';
    case 'dreamSnapSubmission':
      return 'DreamSnaps: Submit a photo';
    case 'dreamSnapVoting':
      return 'DreamSnaps: Vote';
    case 'scroogeStore': {
      const count = item.newItems.length;
      return `Scrooge's Store: ${count} new item${count === 1 ? '' : 's'}`;
    }
  }
}

export function checklistDescription(item: ChecklistItem): string {
  const details =
    item.kind === 'scroogeStore'
      ? item.newItems.map((entry) => entry.name || `Item ${entry.id}`).join(', ')
      : null;
  return [item.location, details].filter(Boolean).join(' · ');
}
