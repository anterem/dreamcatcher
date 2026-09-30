import { describe, it, expect } from 'vitest';
import { localWeekday, upcomingWindowStart } from './time';

const H = 3600;
const DAY = 86400;

const SUNDAY = 3 * DAY;
const MONDAY = 4 * DAY;
const TUESDAY = 5 * DAY;

const at = (base: number, hour: number) => base + hour * H;

describe('localWeekday', () => {
  it('maps the epoch (Thursday) to index 4', () => {
    expect(localWeekday(0, 0)).toBe(4);
  });

  it('numbers days from Sunday', () => {
    expect(localWeekday(SUNDAY, 0)).toBe(0);
    expect(localWeekday(MONDAY, 0)).toBe(1);
    expect(localWeekday(TUESDAY, 0)).toBe(2);
  });

  it('shifts the day across midnight by timezone', () => {
    expect(localWeekday(at(SUNDAY, 23), 2 * H)).toBe(1);
    expect(localWeekday(at(MONDAY, 0), -H)).toBe(0);
  });
});

describe('upcoming critter availability', () => {
  it('returns the earliest time the critter becomes available', () => {
    const dual = [
      { start: 7, end: 8 },
      { start: 19, end: 20 }
    ];
    expect(upcomingWindowStart(dual, 6)).toBe(7);
    expect(upcomingWindowStart(dual, 12)).toBe(19);
  });

  it('returns null when it is not available any more today', () => {
    expect(upcomingWindowStart([{ start: 7, end: 8 }], 8)).toBeNull();
    expect(upcomingWindowStart([{ start: 0, end: 24 }], 5)).toBeNull();
    expect(upcomingWindowStart([], 0)).toBeNull();
  });
});
