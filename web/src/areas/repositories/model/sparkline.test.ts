import { describe, expect, it } from 'vitest';
import { sparklinePath } from './sparkline';

describe('sparklinePath', () => {
  it('draws nothing for an empty or a single-point series', () => {
    expect(sparklinePath([], 96, 24)).toBe('');
    expect(sparklinePath([7], 96, 24)).toBe('');
  });

  it('scales a rising series into the box', () => {
    expect(sparklinePath([1, 2, 3], 20, 10)).toBe('M0.00,10.00 L10.00,5.00 L20.00,0.00');
  });

  it('draws a flat series through the middle', () => {
    expect(sparklinePath([4, 4], 20, 10)).toBe('M0.00,5.00 L20.00,5.00');
  });
});
