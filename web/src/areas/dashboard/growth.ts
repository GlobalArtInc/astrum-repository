import type { UsageSeries } from '../repositories/api';

export const GROWTH_LIMIT = 5;

export interface Growth {
  name: string;
  first: number;
  last: number;
  growth: number;
}

export function growthRanking(series: UsageSeries, limit: number = GROWTH_LIMIT): Growth[] {
  return Object.entries(series)
    .filter(([, values]) => values.length >= 2)
    .map(([name, values]) => {
      const first = values[0] ?? 0;
      const last = values[values.length - 1] ?? 0;
      return { name, first, last, growth: last - first };
    })
    .filter((row) => row.growth > 0)
    .sort((left, right) => right.growth - left.growth || left.name.localeCompare(right.name))
    .slice(0, limit);
}
