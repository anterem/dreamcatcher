import type { Critter } from '../../lib/bindings';
import { localHour, localMidnight, localWeekday } from '../../lib/time';

export type CritterStatus = Critter & {
  availableNow: boolean;
  fedToday: boolean;
  needsFeeding: boolean;
};

function isAvailable(schedule: Critter['schedule'][number], hour: number): boolean {
  return schedule.some((s) => hour >= s.start && hour < s.end);
}

export function critterStatus(critter: Critter, nowSecs: number, tzOffset: number): CritterStatus {
  const availableNow = isAvailable(
    critter.schedule[localWeekday(nowSecs, tzOffset)],
    localHour(nowSecs, tzOffset)
  );
  const fedToday =
    critter.lastFeedingSecs !== null && critter.lastFeedingSecs >= localMidnight(nowSecs, tzOffset);
  return {
    ...critter,
    availableNow,
    fedToday,
    needsFeeding: availableNow && !fedToday
  };
}

