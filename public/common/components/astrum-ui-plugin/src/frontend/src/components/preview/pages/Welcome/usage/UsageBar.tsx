import React from 'react';

export interface UsageBarProps {
  /** Share of the bar to fill, 0 to 1. */
  share: number;
  color?: 'accent' | 'orange';
  className?: string;
}

/**
 * A thin horizontal bar showing a value as a share of the largest one in its list; the number next to it carries the
 * value, the bar only its size.
 */
export function UsageBar({ share, color = 'accent', className }: UsageBarProps): JSX.Element {
  const width = `${Math.max(0, Math.min(1, share)) * 100}%`;
  return (
    <span
      className={['as-usage-bar', `as-usage-bar--${color}`, className].filter(Boolean).join(' ')}
      aria-hidden="true"
    >
      <span className="as-usage-bar__fill" style={{ width }} />
    </span>
  );
}

export default UsageBar;
