import { useEffect, useState, useSyncExternalStore } from 'react';

export const PRODUCT = 'Astrum Repository';
export const TITLE_SUFFIX = 'GlobalArt Astrum Repository';
export const PHONE = '(max-width: 639px)';
export const COMPACT = '(max-width: 1023px)';

function subscribe(query: string) {
  return (notify: () => void) => {
    const list = window.matchMedia(query);
    list.addEventListener('change', notify);
    return () => list.removeEventListener('change', notify);
  };
}

export function useMediaQuery(query: string): boolean {
  return useSyncExternalStore(
    subscribe(query),
    () => window.matchMedia(query).matches,
    () => false,
  );
}

let titleSuffix = TITLE_SUFFIX;

export function documentTitle(title?: string): string {
  return title ? `${title} - ${titleSuffix}` : titleSuffix;
}

export function replaceTitleSuffix(next?: string | null): void {
  const wanted = next?.trim() ? next.trim() : TITLE_SUFFIX;
  if (wanted === titleSuffix) {
    return;
  }
  const previous = titleSuffix;
  titleSuffix = wanted;
  if (document.title === previous) {
    document.title = wanted;
  } else if (document.title.endsWith(` - ${previous}`)) {
    document.title = `${document.title.slice(0, -previous.length)}${wanted}`;
  }
}

export function useDocumentTitle(title?: string): void {
  useEffect(() => {
    document.title = documentTitle(title);
  }, [title]);
}

export function useNow(intervalMs: number): Date {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), intervalMs);
    return () => window.clearInterval(timer);
  }, [intervalMs]);
  return now;
}

export function readStored(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

export function writeStored(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value);
  } catch {}
}
