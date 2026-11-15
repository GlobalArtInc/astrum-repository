import { describe, expect, it } from 'vitest';
import {
  blocksEntered,
  COOLDOWN_MAX_DAYS,
  cooldownPatternProblem,
  cooldownProblems,
  EMPTY_FORM,
  formOf,
  idsOf,
  type Policy,
  requestOf,
  validDays,
} from './model';

const saved: Policy = {
  name: 'strict',
  notes: '',
  vulnerabilities: { mode: 'report', severity: 'high', malicious: 'block', unscanned: 'allow', notify: null },
  licenses: { mode: 'off', list: null, ids: [], unknown: 'report' },
  signatures: { mode: 'off', keys: [] },
  revision: 3,
};

describe('policy forms', () => {
  it('sends the three blocks as the REST reads them and the name only when creating', () => {
    const form = {
      ...formOf(saved),
      licenseMode: 'block' as const,
      licenseList: 'deny',
      licenseIds: 'GPL-3.0-only, AGPL-3.0\nSSPL-1.0',
    };
    expect(requestOf(form, false)).toEqual({
      notes: '',
      vulnerabilities: {
        mode: 'report',
        severity: 'high',
        malicious: 'block',
        unscanned: 'allow',
        notify: null,
      },
      licenses: {
        mode: 'block',
        list: 'deny',
        ids: ['GPL-3.0-only', 'AGPL-3.0', 'SSPL-1.0'],
        unknown: 'report',
      },
      signatures: { mode: 'off', keys: [] },
      cooldown: { mode: 'off', minAgeDays: 7, exclude: [] },
    });
    expect(requestOf({ ...EMPTY_FORM, name: 'fresh' }, true).name).toBe('fresh');
  });

  it('sends the cooldown block with its days and excluded packages, and reads a missing block as off', () => {
    const form = {
      ...formOf(saved),
      cooldownMode: 'block' as const,
      minAgeDays: 14,
      exclude: ['@globalart/*'],
    };
    expect(requestOf(form, false).cooldown).toEqual({
      mode: 'block',
      minAgeDays: 14,
      exclude: ['@globalart/*'],
    });
    expect(formOf(saved)).toMatchObject({ cooldownMode: 'off', minAgeDays: 7, exclude: [] });
  });

  it('checks the cooldown days and the excluded list against the ranges and lengths', () => {
    expect([1, COOLDOWN_MAX_DAYS].every((days) => validDays(days))).toBe(true);
    expect([0, 91, 1.5, null].some((days) => validDays(days))).toBe(false);
    const blocked = { ...formOf(saved), cooldownMode: 'block' as const, minAgeDays: 91 };
    expect(cooldownProblems(blocked)).toEqual({ minAgeDays: 'security.policy.cooldown.daysRange' });
    expect(cooldownProblems({ ...blocked, cooldownMode: 'off' })).toEqual({});
    expect(cooldownProblems({ ...formOf(saved), exclude: ['a'.repeat(201)] })).toEqual({
      exclude: 'security.policy.cooldown.excludeLength',
    });
    expect(cooldownPatternProblem('a', Array(100).fill('b'))).toBe('security.policy.cooldown.excludeCount');
    expect(cooldownPatternProblem('a'.repeat(200), [])).toBeNull();
  });

  it('asks for a confirmation only when a block is switched on', () => {
    const blocked = { ...formOf(saved), vulnerabilityMode: 'block' as const };
    expect(blocksEntered(formOf(saved), blocked)).toEqual(['vulnerabilities']);
    expect(blocksEntered(blocked, blocked)).toEqual([]);
    expect(idsOf('  MIT\tBSD-3-Clause ,ISC ')).toEqual(['MIT', 'BSD-3-Clause', 'ISC']);
  });
});
