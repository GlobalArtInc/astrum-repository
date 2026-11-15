import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderWithApp } from '../../../test/render';
import type { RepositoryUsage } from '../api';
import { UsageBar } from './UsageBar';

function usage(bytes: number, percentUsed: number | null, limitBytes: number | null = 1000): RepositoryUsage {
  return {
    assets: 3,
    bytes,
    quota: limitBytes === null ? null : { limitBytes, warnPercent: 90, enforce: true },
    percentUsed,
  };
}

function level(): string | null | undefined {
  return screen.getByRole('progressbar').firstElementChild?.getAttribute('data-level');
}

describe('the usage bar', () => {
  it('shows the bytes, and a bar only with a quota, in the normal, warning and full states', () => {
    const none = renderWithApp(<UsageBar usage={usage(2048, null, null)} />);
    expect(screen.queryByRole('progressbar')).toBeNull();
    expect(screen.getByText('No quota is set; the size is still counted')).toBeTruthy();
    none.unmount();

    const normal = renderWithApp(<UsageBar usage={usage(250, 25)} />);
    expect(screen.getByRole('progressbar').getAttribute('aria-valuenow')).toBe('25');
    expect(level()).toBe('normal');
    expect(screen.queryByText('At or above the warning level')).toBeNull();
    normal.unmount();

    const warning = renderWithApp(<UsageBar usage={usage(950, 95)} />);
    expect(level()).toBe('warning');
    expect(screen.getByText('95.0% of the limit')).toBeTruthy();
    expect(screen.getByText('At or above the warning level')).toBeTruthy();
    warning.unmount();

    renderWithApp(<UsageBar usage={usage(1500, 150)} />);
    expect(level()).toBe('full');
    expect(screen.getByRole('progressbar').getAttribute('aria-valuenow')).toBe('100');
    expect(screen.getByText('At or over the limit')).toBeTruthy();
  });
});
