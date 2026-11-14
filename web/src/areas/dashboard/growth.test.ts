import { describe, expect, it } from 'vitest';
import { growthRanking } from './growth';

describe('growthRanking', () => {
  it('ranks the positive growth between the first and the last point', () => {
    const ranking = growthRanking(
      {
        flat: [5, 5],
        shrinking: [9, 1],
        single: [7],
        empty: [],
        small: [1, 3],
        big: [0, 10, 4, 20],
        mid: [2, 9],
      },
      2,
    );
    expect(ranking).toEqual([
      { name: 'big', first: 0, last: 20, growth: 20 },
      { name: 'mid', first: 2, last: 9, growth: 7 },
    ]);
  });

  it('lists five repositories by default', () => {
    const series = Object.fromEntries(
      Array.from({ length: 6 }, (_, index) => [`repo-${index}`, [0, index + 1]]),
    );
    expect(growthRanking(series).map((row) => row.name)).toEqual([
      'repo-5',
      'repo-4',
      'repo-3',
      'repo-2',
      'repo-1',
    ]);
  });
});
