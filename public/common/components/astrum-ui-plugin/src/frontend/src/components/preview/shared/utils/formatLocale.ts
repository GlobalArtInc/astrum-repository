import { pageLanguage } from '../../../../interface/plural';

/** Formats a time of day the way the page's language writes it, e.g. `3:00 AM` in English and `3:00` in Russian. */
export function formatClockTime(hours: number, minutes: number): string {
  return new Intl.DateTimeFormat(pageLanguage(), { hour: 'numeric', minute: '2-digit', timeZone: 'UTC' })
    .format(Date.UTC(2000, 0, 1, hours, minutes))
    .replace(/ /g, ' ');
}
