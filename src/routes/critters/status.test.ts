import { describe, it, expect } from 'vitest';
import { critterStatus } from './status';
import type { Critter, Schedule } from '$lib/bindings';

const H = 3600;
const DAY = 86400;

const SUNDAY = 3 * DAY;

const at = (base: number, hour: number) => base + hour * H;

function schedule(perDay: Record<number, Schedule[]>): Schedule[][] {
  return Array.from({ length: 7 }, (_, day) => perDay[day] ?? []);
}

function critter(overrides: Partial<Critter> = {}): Critter {
  return {
    itemId: 1,
    name: 'Test Critter',
    species: 'squirrel',
    speciesRank: 0,
    biome: 'Peaceful Meadow',
    notes: [],
    schedule: schedule({}),
    tamed: false,
    lastFeedingSecs: null,
    ...overrides
  };
}

const availableAt = (s: Record<number, Schedule[]>, now: number) =>
  critterStatus(critter({ schedule: schedule(s) }), now, 0).availableNow;

describe('critter availability', () => {
  it('an empty schedule is never available', () => {
    expect(availableAt({}, at(SUNDAY, 12))).toBe(false);
  });

  it('a full-day window covers every hour', () => {
    const all = { [0]: [{ start: 0, end: 24 }] };
    expect(availableAt(all, at(SUNDAY, 0))).toBe(true);
    expect(availableAt(all, at(SUNDAY, 12))).toBe(true);
    expect(availableAt(all, at(SUNDAY, 23))).toBe(true);
  });

  it('a morning window spans midnight to noon, end-exclusive', () => {
    const am = { [0]: [{ start: 0, end: 12 }] };
    expect(availableAt(am, at(SUNDAY, 0))).toBe(true);
    expect(availableAt(am, at(SUNDAY, 11))).toBe(true);
    expect(availableAt(am, at(SUNDAY, 12))).toBe(false);
    expect(availableAt(am, at(SUNDAY, 23))).toBe(false);
  });

  it('an afternoon window spans noon to midnight', () => {
    const pm = { [0]: [{ start: 12, end: 24 }] };
    expect(availableAt(pm, at(SUNDAY, 11))).toBe(false);
    expect(availableAt(pm, at(SUNDAY, 12))).toBe(true);
    expect(availableAt(pm, at(SUNDAY, 23))).toBe(true);
  });

  it('respects custom window boundaries', () => {
    const window = { [0]: [{ start: 15, end: 20 }] };
    expect(availableAt(window, at(SUNDAY, 14))).toBe(false);
    expect(availableAt(window, at(SUNDAY, 15))).toBe(true);
    expect(availableAt(window, at(SUNDAY, 19))).toBe(true);
    expect(availableAt(window, at(SUNDAY, 20))).toBe(false);
  });

  it('handles two windows in one day', () => {
    const dual = {
      [0]: [
        { start: 7, end: 8 },
        { start: 19, end: 20 }
      ]
    };
    expect(availableAt(dual, at(SUNDAY, 7))).toBe(true);
    expect(availableAt(dual, at(SUNDAY, 8))).toBe(false);
    expect(availableAt(dual, at(SUNDAY, 12))).toBe(false);
    expect(availableAt(dual, at(SUNDAY, 19))).toBe(true);
    expect(availableAt(dual, at(SUNDAY, 20))).toBe(false);
  });

  it('gates by day of week', () => {
    const tuesdayOnly = { [2]: [{ start: 0, end: 24 }] };
    expect(availableAt(tuesdayOnly, at(SUNDAY, 12))).toBe(false);
    expect(availableAt(tuesdayOnly, at(SUNDAY + DAY, 12))).toBe(false);
    expect(availableAt(tuesdayOnly, at(SUNDAY + 2 * DAY, 12))).toBe(true);
  });
});

describe('critter feeding', () => {
  const dayLong = { [0]: [{ start: 0, end: 24 }] };
  const status = (lastFeedingSecs: number | null, tamed = false) =>
    critterStatus(
      critter({ schedule: schedule(dayLong), tamed, lastFeedingSecs }),
      at(SUNDAY, 12),
      0
    );

  it('is unfed when never fed', () => {
    expect(status(null).fedToday).toBe(false);
  });

  it('counts a feeding from today', () => {
    expect(status(SUNDAY).fedToday).toBe(true);
    expect(status(SUNDAY - 1).fedToday).toBe(false);
  });

  it('needs feeding when available and unfed, whether or not tamed', () => {
    expect(status(null).needsFeeding).toBe(true);
    expect(status(SUNDAY).needsFeeding).toBe(false);
    expect(status(null, true).needsFeeding).toBe(true);
    expect(status(SUNDAY, true).needsFeeding).toBe(false);
  });
});
