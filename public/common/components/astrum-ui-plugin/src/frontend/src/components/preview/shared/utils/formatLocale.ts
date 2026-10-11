import { pageLanguage } from '../../../../interface/plural';

/** Formats a time of day the way the page's language writes it, e.g. `3:00 AM` in English and `3:00` in Russian. */
export function formatClockTime(hours: number, minutes: number): string {
  return new Intl.DateTimeFormat(pageLanguage(), { hour: 'numeric', minute: '2-digit', timeZone: 'UTC' })
    .format(Date.UTC(2000, 0, 1, hours, minutes))
    .replace(/\u202f/g, ' ');
}

/** Formats a number of a unit of time with the page language's plural forms, e.g. `30 days` or `30 дней`. */
export function formatDuration(value: number, unit: 'second' | 'minute' | 'hour' | 'day'): string {
  try {
    return new Intl.NumberFormat(pageLanguage(), { style: 'unit', unit, unitDisplay: 'long' }).format(value);
  }
  catch {
    return `${value} ${unit}${value === 1 ? '' : 's'}`;
  }
}
