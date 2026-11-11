import { describe, expect, it } from 'vitest';
import { blocksEntered, EMPTY_FORM, formOf, idsOf, type Policy, requestOf } from './model';

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
    });
    expect(requestOf({ ...EMPTY_FORM, name: 'fresh' }, true).name).toBe('fresh');
  });

  it('asks for a confirmation only when a block is switched on', () => {
    const blocked = { ...formOf(saved), vulnerabilityMode: 'block' as const };
    expect(blocksEntered(formOf(saved), blocked)).toEqual(['vulnerabilities']);
    expect(blocksEntered(blocked, blocked)).toEqual([]);
    expect(idsOf('  MIT\tBSD-3-Clause ,ISC ')).toEqual(['MIT', 'BSD-3-Clause', 'ISC']);
  });
});
