import type { Schedule } from './bindings';

const DAY = 86400;
const HOUR = 3600;

function localSecs(nowSecs: number, tzOffset: number): number {
  return nowSecs + tzOffset;
}

// 0 = Sunday, matches critter schedule array
export function localWeekday(nowSecs: number, tzOffset: number): number {
  const days = Math.floor(localSecs(nowSecs, tzOffset) / DAY);
  return (((days + 4) % 7) + 7) % 7;
}

export function localHour(nowSecs: number, tzOffset: number): number {
  const secs = ((localSecs(nowSecs, tzOffset) % DAY) + DAY) % DAY;
  return Math.floor(secs / HOUR);
}

export function localMidnight(nowSecs: number, tzOffset: number): number {
  return Math.floor(localSecs(nowSecs, tzOffset) / DAY) * DAY;
}

// gifts reset at 5am local
export function last5amUtc(nowSecs: number, tzOffset: number): number {
  const local = localSecs(nowSecs, tzOffset);
  const local5am = Math.floor((local - 5 * HOUR) / DAY) * DAY + 5 * HOUR;
  return local5am - tzOffset;
}

export function upcomingWindowStart(schedule: Schedule[], hour: number): number | null {
  const upcoming = schedule.filter((s) => s.start > hour).map((s) => s.start);
  return upcoming.length ? Math.min(...upcoming) : null;
}
