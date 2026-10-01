import type { ChecklistFacts } from './bindings';

type ChecklistItem = { label: string; description?: string | null };

export function checklistItems(facts: ChecklistFacts): ChecklistItem[] {
  const items: ChecklistItem[] = facts.moonstoneChestBiomes.map((biome) => ({
    label: 'Daily Moonstone Chest',
    description: biome
  }));
  for (const biome of facts.riftBiomes) items.push({ label: 'Time Rift', description: biome });
  if (facts.dreamSnaps?.submitNeeded) items.push({ label: 'DreamSnaps: Submit a photo' });
  if (facts.dreamSnaps?.voteNeeded) items.push({ label: 'DreamSnaps: Vote' });

  for (const store of facts.scroogeStores) {
    const count = store.newItems.length;
    const names = store.newItems.map((entry) => entry.name || `Item ${entry.id}`).join(', ');
    items.push({
      label: `Scrooge's Store: ${count} new item${count === 1 ? '' : 's'}`,
      description: [store.location, names].filter(Boolean).join(' · ')
    });
  }

  return items;
}
