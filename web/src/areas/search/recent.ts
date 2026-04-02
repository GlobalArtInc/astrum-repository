import { readStored, writeStored } from '../../ui';

export const RECENT_KEY = 'astrum-recent-searches';
export const RECENT_LIMIT = 5;

export function readRecent(): string[] {
  const stored = readStored(RECENT_KEY);
  if (!stored) {
    return [];
  }
  try {
    const parsed: unknown = JSON.parse(stored);
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed
      .filter((entry): entry is string => typeof entry === 'string' && entry.trim() !== '')
      .slice(0, RECENT_LIMIT);
  } catch {
    return [];
  }
}

export function rememberSearch(query: string): string[] {
  const text = query.trim();
  if (text === '') {
    return readRecent();
  }
  const next = [text, ...readRecent().filter((entry) => entry !== text)].slice(0, RECENT_LIMIT);
  writeStored(RECENT_KEY, JSON.stringify(next));
  return next;
}

export function forgetSearch(query: string): string[] {
  const next = readRecent().filter((entry) => entry !== query);
  writeStored(RECENT_KEY, JSON.stringify(next));
  return next;
}
