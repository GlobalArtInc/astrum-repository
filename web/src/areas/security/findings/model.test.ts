import { describe, expect, it } from 'vitest';
import { ApiError } from '../../../api';
import { isUnavailable, worstSeverity } from './model';

describe('worstSeverity', () => {
  it('ranks critical above high, medium, low and unknown', () => {
    expect(worstSeverity(['low', 'high', 'unknown', 'medium'])).toBe('high');
    expect(worstSeverity(['unknown', 'low'])).toBe('low');
    expect(worstSeverity([])).toBeNull();
  });
});

describe('isUnavailable', () => {
  it('treats 401, 403 and 404 as no data and other failures as errors', () => {
    expect([401, 403, 404].map((status) => isUnavailable(new ApiError(status, 'x')))).toEqual([
      true,
      true,
      true,
    ]);
    expect(isUnavailable(new ApiError(500, 'x'))).toBe(false);
  });
});
