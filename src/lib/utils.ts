import type { Role, Schedule } from './bindings';

const ROLE_LABELS: Record<Role, string> = {
  gardening: 'Gardening',
  fishing: 'Fishing',
  mining: 'Mining',
  foraging: 'Foraging',
  digging: 'Digging',
  timebending: 'Timebending',
  snippetCatching: 'Snippet Catching'
};

export function roleLabel(role: Role | null): string {
  return role ? ROLE_LABELS[role] : '';
}

export const ROLES = Object.keys(ROLE_LABELS) as Role[];

export const ROLE_ICONS: Record<Role, string> = {
  gardening: '🌱',
  foraging: '🫐',
  mining: '⛏️',
  digging: '🟫',
  fishing: '🎣',
  timebending: '⏳',
  snippetCatching: '🕸️'
};

function to12HourTime(h: number): { hour: number; meridiem: 'AM' | 'PM' } {
  return {
    hour: h % 12 === 0 ? 12 : h % 12,
    meridiem: h % 24 < 12 ? 'AM' : 'PM'
  };
}

export function formatHour(hour: number): string {
  const { hour: h, meridiem } = to12HourTime(hour);
  return `${h} ${meridiem}`;
}

function formatHourRange({ start, end }: Schedule): string {
  const s = to12HourTime(start);
  const e = to12HourTime(end);
  return (
    `${s.hour}` + (s.meridiem === e.meridiem ? '' : ` ${s.meridiem}`) + ` – ${e.hour} ${e.meridiem}`
  );
}

export function formatSchedule(daySchedule: Schedule[]): string {
  if (daySchedule.length === 0) return '—';
  if (daySchedule.length === 1) {
    const { start, end } = daySchedule[0];
    if (start === 0 && end === 24) return 'All day';
    if (start === 0 && end === 12) return 'Morning';
    if (start === 12 && end === 24) return 'Afternoon';
  }
  return daySchedule.map(formatHourRange).join(', ');
}

export const WEEKDAY_NAMES = [
  'Sunday',
  'Monday',
  'Tuesday',
  'Wednesday',
  'Thursday',
  'Friday',
  'Saturday'
];

const rtf = new Intl.RelativeTimeFormat(
  typeof navigator !== 'undefined' ? navigator.language : 'en',
  { numeric: 'auto' }
);

export function formatRelativeTime(unixSeconds: number, nowMs = Date.now()): string {
  if (!Number.isFinite(unixSeconds)) return 'unknown';

  const relMs = unixSeconds * 1000 - nowMs;
  const absMs = Math.abs(relMs);

  if (absMs < 60_000) return rtf.format(Math.round(relMs / 1_000), 'second');
  if (absMs < 3_600_000) return rtf.format(Math.round(relMs / 60_000), 'minute');
  if (absMs < 86_400_000) return rtf.format(Math.round(relMs / 3_600_000), 'hour');
  if (absMs < 604_800_000) return rtf.format(Math.round(relMs / 86_400_000), 'day');
  if (absMs < 2_592_000_000) return rtf.format(Math.round(relMs / 604_800_000), 'week');
  if (absMs < 31_536_000_000) return rtf.format(Math.round(relMs / 2_592_000_000), 'month');
  return rtf.format(Math.round(relMs / 31_536_000_000), 'year');
}
