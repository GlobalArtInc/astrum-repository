

import React from 'react';
import { CheckCircle, ChevronRight } from 'lucide-react';
import { FormatBadge } from '../../../shared';
import type { GAResult } from '../core';

import './GAResultRow.scss';

export interface GAResultRowProps {
  /** The GA result to display */
  result: GAResult;
  /** Callback when row is selected */
  onSelect: (gaId: string) => void;
}

/**
 * Formats a date string for display.
 */
function formatDate(isoDate: string): string {
  const date = new Date(isoDate);
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * Single row component for GA search results.
 * Card-style display with colored format icon matching Figma design.
 */
export function GAResultRow({ result, onSelect }: GAResultRowProps): JSX.Element {
  const handleClick = (): void => {
    onSelect(result.gaId);
  };

  const handleKeyDown = (event: React.KeyboardEvent): void => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onSelect(result.gaId);
    }
  };

  return (
    <div
      className="ga-result-row"
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={0}
      role="button"
      aria-label={`View details for ${result.displayName}`}
    >
      <FormatBadge
        format={result.format}
        variant="tile"
        size={24}
        showLabel={false}
        className="ga-result-row__icon-tile"
      />

      <div className="ga-result-row__content">
        <div className="ga-result-row__name">
          <span className="ga-result-row__name-text">{result.displayName}</span>
          <CheckCircle className="ga-result-row__verified" size={14} />
        </div>

        <div className="ga-result-row__namespace">
          {result.namespace}
        </div>

        <div className="ga-result-row__meta">
          <FormatBadge format={result.format} size={14} className="ga-result-row__badge" />
          <span className="ga-result-row__info">
            Latest: {result.latestVersion ?? '-'}
          </span>
          <span className="ga-result-row__info">
            Versions: {result.versionsCount}
          </span>
          <span className="ga-result-row__info">
            Repos: {result.repositoriesCount}
          </span>
        </div>
      </div>

      <div className="ga-result-row__date">
        Updated {formatDate(result.lastUpdated)}
      </div>

      <ChevronRight className="ga-result-row__chevron" size={20} />
    </div>
  );
}

export default GAResultRow;

