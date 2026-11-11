import { DAY } from './model';

export const STALE_AFTER_DAYS = 7;

export interface AdvisoryFeed {
  ecosystem: string;
  records: number;
  syncedAt: string | null;
  attemptedAt: string | null;
  lastError: string | null;
}

export interface ImportResult {
  ecosystem: string;
  records: number;
  changed: number;
  unchanged: number;
  removed: number;
  skipped: number;
}

export function ageInDays(syncedAt: string, now: number): number {
  return Math.floor((now - Date.parse(syncedAt)) / DAY);
}

export function isStale(feed: AdvisoryFeed, now: number): boolean {
  return feed.syncedAt === null || ageInDays(feed.syncedAt, now) > STALE_AFTER_DAYS;
}
