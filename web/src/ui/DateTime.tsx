import { formatDate, formatDateTime, formatRelative, useI18n } from '../i18n';
import { useNow } from './hooks';

type When = Date | string | number;

function iso(value: When): string {
  const date = value instanceof Date ? value : new Date(value);
  return Number.isNaN(date.getTime()) ? '' : date.toISOString();
}

export function DateTime({ value, dateOnly = false }: { value: When; dateOnly?: boolean }) {
  const { language } = useI18n();
  return (
    <time dateTime={iso(value)}>
      {dateOnly ? formatDate(language, value) : formatDateTime(language, value)}
    </time>
  );
}

export function RelativeTime({ value }: { value: When }) {
  const { language } = useI18n();
  const now = useNow(60_000);
  return (
    <time dateTime={iso(value)} title={formatDateTime(language, value)}>
      {formatRelative(language, value, now)}
    </time>
  );
}
