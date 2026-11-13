import { describe, expect, it } from 'vitest';
import type { RepositoryDocument } from './document';
import {
  amountIn,
  bestUnit,
  bytesIn,
  DEFAULT_WARN_PERCENT,
  quotaDocument,
  quotaFields,
  usageLevel,
} from './quota';

describe('quota units', () => {
  it('convert between bytes and the chosen unit', () => {
    expect(amountIn(3 * 1024 ** 3, 'GiB')).toBe(3);
    expect(bytesIn(2, 'MiB')).toBe(2 * 1024 ** 2);
    expect(bytesIn(1.5, 'KiB')).toBe(1536);
  });

  it('show a limit in the largest unit it fills exactly', () => {
    expect(bestUnit(2 * 1024 ** 4)).toBe('TiB');
    expect(bestUnit(5 * 1024 ** 2)).toBe('MiB');
    expect(bestUnit(1536)).toBe('KiB');
    expect(bestUnit(1000)).toBe('KiB');
    expect(bestUnit(null)).toBe('GiB');
  });
});

describe('the storage.quota object', () => {
  it('is absent without a limit, and carries the three settings with one', () => {
    expect(quotaDocument({ limitBytes: null, warnPercent: 90, enforce: true })).toBeNull();
    expect(quotaDocument({ limitBytes: 100, warnPercent: 80, enforce: true })).toEqual({
      limitBytes: 100,
      warnPercent: 80,
      enforce: true,
    });
  });

  it('is read with the default warning percent when none is stored', () => {
    const stored: RepositoryDocument = { storage: { quota: { limitBytes: 100, enforce: false } } };
    expect(quotaFields(stored)).toEqual({
      limitBytes: 100,
      warnPercent: DEFAULT_WARN_PERCENT,
      enforce: false,
    });
    expect(quotaFields({ storage: {} })).toEqual({ limitBytes: null, warnPercent: 90, enforce: false });
  });
});

describe('the usage level', () => {
  it('is normal below the warning percent, warning from it, and full from 100', () => {
    expect(usageLevel(89.9, 90)).toBe('normal');
    expect(usageLevel(90, 90)).toBe('warning');
    expect(usageLevel(99.9, 90)).toBe('warning');
    expect(usageLevel(100, 90)).toBe('full');
    expect(usageLevel(150, 90)).toBe('full');
  });
});
