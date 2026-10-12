import UIStrings from '../../../../../constants/UIStrings';
import { pageLanguage } from '../../../../../interface/plural';
import { FORMAT_LABELS } from '../../../shared/Badges/formatConstants';
import type { UsageComponent } from './usageDashboard.types';

const BYTE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const;

function units(): Record<string, string> {
  return UIStrings.USAGE_DASHBOARD.UNITS;
}

/** Formats a count the way the page's language writes numbers. */
export function formatNumber(value: number, maximumFractionDigits = 0): string {
  return new Intl.NumberFormat(pageLanguage(), { maximumFractionDigits }).format(value);
}

/** Formats a size in bytes with binary multiples and one decimal, e.g. `186.4 GB`. */
export function formatBytes(bytes: number): string {
  let value = Math.max(0, bytes);
  let unit = 0;
  while (value >= 1024 && unit < BYTE_UNITS.length - 1) {
    value /= 1024;
    unit++;
  }
  const digits = unit === 0 || value >= 100 ? 0 : 1;
  return `${formatNumber(value, digits)} ${units()[BYTE_UNITS[unit]]}`;
}

/** Formats a number of days, e.g. `30 days`, with the page language's plural forms. */
export function formatDays(days: number): string {
  try {
    return new Intl.NumberFormat(pageLanguage(), { style: 'unit', unit: 'day', unitDisplay: 'long' }).format(days);
  }
  catch {
    return `${days} days`;
  }
}

function utcDate(day: string): Date {
  return new Date(`${day}T00:00:00Z`);
}

/** Whether a UTC day, `yyyy-MM-dd`, is a Saturday or Sunday. */
export function isWeekend(day: string): boolean {
  const weekday = utcDate(day).getUTCDay();
  return weekday === 0 || weekday === 6;
}

/** Formats a UTC day as day and short month, e.g. `Oct 11`. */
export function formatDay(day: string): string {
  return new Intl.DateTimeFormat(pageLanguage(), { day: 'numeric', month: 'short', timeZone: 'UTC' })
    .format(utcDate(day));
}

/** Formats a UTC day as short weekday and day, e.g. `Sat 11`. */
export function formatWeekday(day: string): string {
  const date = utcDate(day);
  const weekday = new Intl.DateTimeFormat(pageLanguage(), { weekday: 'short', timeZone: 'UTC' }).format(date);
  return `${weekday} ${date.getUTCDate()}`;
}

/** Formats a UTC day with its weekday, for tooltips, e.g. `Saturday, October 11`. */
export function formatLongDay(day: string): string {
  return new Intl.DateTimeFormat(pageLanguage(), {
    weekday: 'long', day: 'numeric', month: 'long', timeZone: 'UTC',
  }).format(utcDate(day));
}

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** Formats how long ago an instant was, e.g. `12 minutes ago` or `yesterday`; older ones as a date. */
export function formatAgo(instant: string, now: number = Date.now()): string {
  const time = new Date(instant).getTime();
  if (Number.isNaN(time)) {
    return '';
  }
  const elapsed = Math.max(0, now - time);
  const language = pageLanguage();
  try {
    const relative = new Intl.RelativeTimeFormat(language, { numeric: 'auto' });
    if (elapsed < MINUTE) {
      return relative.format(0, 'second');
    }
    if (elapsed < HOUR) {
      return relative.format(-Math.floor(elapsed / MINUTE), 'minute');
    }
    if (elapsed < DAY) {
      return relative.format(-Math.floor(elapsed / HOUR), 'hour');
    }
    if (elapsed < 7 * DAY) {
      return relative.format(-Math.floor(elapsed / DAY), 'day');
    }
  }
  catch {
    // fall through to the date
  }
  return new Intl.DateTimeFormat(language, { day: 'numeric', month: 'short', year: 'numeric' }).format(new Date(time));
}

/** The display name of a format, e.g. `Maven` for `maven2`. */
export function formatLabel(format: string): string {
  return FORMAT_LABELS[format] || format;
}

/**
 * The name of a component as its format writes it: Maven `group:name`, npm `@scope/name`, else `group/name`; names
 * that are paths, like those of raw components, without their leading slash.
 */
export function componentName(component: Pick<UsageComponent, 'format' | 'group' | 'name'>): string {
  const { format, group, name } = component;
  if (name.startsWith('/')) {
    return name.slice(1);
  }
  if (!group) {
    return name;
  }
  if (format === 'maven2') {
    return `${group}:${name}`;
  }
  if (format === 'npm') {
    return `@${group}/${name}`;
  }
  return `${group}/${name}`;
}

function encodePath(path: string): string {
  return path.split('/').filter(Boolean).map(encodeURIComponent).join('/');
}

/** The browse tree path of a component: its node when the server knows it, else one built from its coordinates. */
export function browsePathOf(component: UsageComponent): string {
  if (component.browsePath) {
    return component.browsePath;
  }
  const { format, group, name, version } = component;
  if (format === 'maven2') {
    return [group?.replace(/\./g, '/'), name, version].filter(Boolean).join('/');
  }
  if (format === 'npm') {
    return [group ? `@${group}` : null, name, version].filter(Boolean).join('/');
  }
  if (format === 'docker') {
    return ['v2', name, 'tags', version].filter(Boolean).join('/');
  }
  return [group, name, version].filter(Boolean).join('/');
}

/** Where the links of the dashboard lead to. */
export const usageLinks = {
  component(component: UsageComponent): string {
    return `#browse/${encodeURIComponent(component.repository)}/${encodePath(browsePathOf(component))}`;
  },
  repository(repository: string): string {
    return `#browse/${encodeURIComponent(repository)}/`;
  },
  search(): string {
    return '#browse/search';
  },
  cleanupPolicies(): string {
    return '#admin/repository/cleanup-policies';
  },
};
