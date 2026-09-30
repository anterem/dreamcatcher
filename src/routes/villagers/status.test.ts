import { describe, it, expect } from 'vitest';
import { villagerStatus } from './status';
import type { PreferredGift, Villager } from '$lib/bindings';

const H = 3600;
const DAY = 86400;

function gift(overrides: Partial<PreferredGift> = {}): PreferredGift {
  return {
    itemId: 1,
    name: 'Apple',
    category: 'produce',
    discovered: true,
    gifted: false,
    ...overrides
  };
}

function villager(overrides: Partial<Villager> = {}): Villager {
  return {
    id: 1,
    name: 'Goofy',
    role: null,
    friendshipLevel: 1,
    friendshipXp: 0,
    isMaxed: false,
    gifts: [],
    ...overrides
  };
}

describe('villager gifts', () => {
  // gifts reset at 5am local time
  const giftable = (v: Villager, modifiedSecs: number, now: number, tz: number) =>
    villagerStatus(v, modifiedSecs, now, tz).giftable;

  const withGift = villager({ gifts: [gift()] });

  it('gifts are valid until reset', () => {
    // written after this morning's 5am
    expect(giftable(withGift, 5 * H, 6 * H, 0)).toBe(true);
    expect(giftable(withGift, 6 * H, 6 * H, 0)).toBe(true);
    // written at exactly 5am
    expect(giftable(withGift, 5 * H, 5 * H, 0)).toBe(true);
    // written before this morning's 5am
    expect(giftable(withGift, 5 * H - 1, 6 * H, 0)).toBe(false);
    // before 5am, yesterday's 5am onwards still counts
    const yesterday5am = 5 * H - DAY;
    expect(giftable(withGift, yesterday5am, 3 * H, 0)).toBe(true);
    expect(giftable(withGift, yesterday5am - 1, 3 * H, 0)).toBe(false);
  });

  it('reset uses local time', () => {
    const tz = 10 * H;
    const now = -tz + 2 * H;
    const cutoff = 5 * H - DAY - tz;
    expect(giftable(withGift, cutoff - 1, now, tz)).toBe(false);
    expect(giftable(withGift, cutoff, now, tz)).toBe(true);
  });

  it('giftable until every preferred gift is given', () => {
    expect(giftable(villager({ gifts: [gift()] }), 6 * H, 6 * H, 0)).toBe(true);
    expect(giftable(villager({ gifts: [] }), 6 * H, 6 * H, 0)).toBe(false);
    expect(giftable(villager({ gifts: [gift({ gifted: true })] }), 6 * H, 6 * H, 0)).toBe(false);
    expect(giftable(villager({ gifts: [gift({ gifted: true }), gift()] }), 6 * H, 6 * H, 0)).toBe(
      true
    );

    const allGifted = villager({ gifts: [gift({ gifted: true })] });
    expect(villagerStatus(allGifted, 6 * H, 6 * H, 0).giftsAreCurrent).toBe(true);
    expect(villagerStatus(allGifted, 5 * H - 1, 6 * H, 0).giftsAreCurrent).toBe(false);
  });
});
