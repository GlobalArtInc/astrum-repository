import type { Language } from './translate';

const ENGLISH_UNITS: readonly string[] = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];

const UNITS: Partial<Record<Language, readonly string[]>> = {
  en: ENGLISH_UNITS,
  ru: ['Б', 'КБ', 'МБ', 'ГБ', 'ТБ', 'ПБ'],
};

export function formatNumber(language: Language, value: number): string {
  return new Intl.NumberFormat(language).format(value);
}

export function formatBytes(language: Language, bytes: number): string {
  const units = UNITS[language] ?? ENGLISH_UNITS;
  let value = Math.max(0, bytes);
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const number = new Intl.NumberFormat(language, {
    maximumFractionDigits: unit === 0 || value >= 100 ? 0 : 1,
  }).format(value);
  return `${number}\u00a0${units[unit]}`;
}

function toDate(value: Date | string | number): Date {
  return value instanceof Date ? value : new Date(value);
}

export function formatDate(language: Language, value: Date | string | number): string {
  return new Intl.DateTimeFormat(language, { dateStyle: 'medium' }).format(toDate(value));
}

export function formatDateTime(language: Language, value: Date | string | number): string {
  return new Intl.DateTimeFormat(language, { dateStyle: 'medium', timeStyle: 'short' }).format(toDate(value));
}

const STEPS: Array<[Intl.RelativeTimeFormatUnit, number]> = [
  ['second', 60],
  ['minute', 60],
  ['hour', 24],
  ['day', 7],
  ['week', 4.348],
  ['month', 12],
  ['year', Number.POSITIVE_INFINITY],
];

export function formatRelative(
  language: Language,
  value: Date | string | number,
  now: Date = new Date(),
): string {
  let delta = (toDate(value).getTime() - now.getTime()) / 1000;
  const format = new Intl.RelativeTimeFormat(language, { numeric: 'auto' });
  for (const [unit, size] of STEPS) {
    if (Math.abs(delta) < size) {
      return format.format(Math.round(delta), unit);
    }
    delta /= size;
  }
  return format.format(Math.round(delta), 'year');
}
