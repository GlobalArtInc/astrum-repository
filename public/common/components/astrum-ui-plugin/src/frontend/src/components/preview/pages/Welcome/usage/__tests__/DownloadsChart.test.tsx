import React from 'react';
import { render } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { DownloadsChart } from '../DownloadsChart';

jest.mock('recharts', () => {
  const Actual = jest.requireActual('recharts');
  const { cloneElement } = jest.requireActual('react');
  return {
    ...Actual,
    ResponsiveContainer: ({ children }: { children: React.ReactElement }) => cloneElement(children,
      { width: 800, height: 260 }),
  };
});

const WEEK = [
  { day: '2026-10-05', downloads: 11 },
  { day: '2026-10-06', downloads: 0 },
  { day: '2026-10-07', downloads: 4 },
  { day: '2026-10-08', downloads: 0 },
  { day: '2026-10-09', downloads: 25 },
  { day: '2026-10-10', downloads: 3 },
  { day: '2026-10-11', downloads: 5 },
];

describe('DownloadsChart', () => {
  it('draws a bar per day, weekends lighter, and labels the peak', () => {
    const { container } = render(<Theme><DownloadsChart days={WEEK} /></Theme>);

    const bars = container.querySelectorAll('.recharts-bar-rectangle path.recharts-rectangle');
    expect(bars.length).toBeGreaterThanOrEqual(5);
    const fills = Array.from(bars).map((bar) => bar.getAttribute('fill'));
    expect(fills).toContain('var(--accent-9)');
    expect(fills).toContain('var(--accent-7)');
    const peak = container.querySelector('.as-usage-chart__peak');
    expect(peak?.textContent).toBe('25');
    expect(container.textContent).toContain('Fri 9');
    expect(container.textContent).toContain('Sun 11');
  });
});
