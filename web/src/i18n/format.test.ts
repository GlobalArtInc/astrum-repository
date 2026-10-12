import { describe, expect, it } from 'vitest';
import { formatBytes, formatDate, formatRelative } from './format';

describe('formatting', () => {
  it('shows sizes in binary units', () => {
    expect(formatBytes('en', 0)).toBe('0\u00a0B');
    expect(formatBytes('en', 1536)).toBe('1.5\u00a0KB');
    expect(formatBytes('en', 5 * 1024 * 1024 * 1024)).toBe('5\u00a0GB');
    expect(formatBytes('en', 150 * 1024 * 1024)).toBe('150\u00a0MB');
    expect(formatBytes('ru', 1536)).toBe('1,5\u00a0КБ');
  });

  it('says how long ago something happened', () => {
    const now = new Date('2026-10-12T12:00:00Z');
    expect(formatRelative('en', new Date('2026-10-12T11:59:30Z'), now)).toBe('30 seconds ago');
    expect(formatRelative('en', new Date('2026-10-12T09:00:00Z'), now)).toBe('3 hours ago');
    expect(formatRelative('en', new Date('2026-10-11T12:00:00Z'), now)).toBe('yesterday');
    expect(formatRelative('ru', new Date('2026-10-10T12:00:00Z'), now)).toBe('позавчера');
    expect(formatRelative('en', new Date('2024-10-12T12:00:00Z'), now)).toBe('2 years ago');
  });

  it('formats dates for the language', () => {
    expect(formatDate('en', '2026-10-12T12:00:00Z')).toMatch(/Oct 12, 2026/);
    expect(formatDate('ru', '2026-10-12T12:00:00Z')).toMatch(/12 окт\. 2026/);
  });
});
