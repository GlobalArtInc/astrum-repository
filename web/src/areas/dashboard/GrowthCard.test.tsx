import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderWithApp } from '../../test/render';
import { GrowthCard } from './GrowthCard';

describe('GrowthCard', () => {
  it('lists the growing repositories with their range and a link to each', () => {
    renderWithApp(
      <GrowthCard
        series={{ alpha: [1024, 2048, 3072], steady: [4096, 4096], gamma: [1024, 10240], shrinking: [9, 1] }}
      />,
    );
    const section = screen.getByRole('region', { name: 'Largest growth, 30 days' });
    const rows = within(section).getAllByRole('listitem');
    expect(rows.map((row) => row.textContent?.replace(/\s/g, ' '))).toEqual([
      'gamma+9 KB1 KB to 10 KB',
      'alpha+2 KB1 KB to 3 KB',
    ]);
    expect(within(section).getByRole('link', { name: 'gamma' })).toHaveAttribute(
      'href',
      '/admin/repository/repositories/gamma',
    );
  });
});
