

import React from 'react';
import type { NpmResult } from './npm.types';

import './NpmResultRow.scss';

export interface NpmResultRowProps {
  /** The npm result to display */
  result: NpmResult;
  /** Callback when row is selected */
  onSelect: (id: string) => void;
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
 * Truncates text to a maximum length.
 */
function truncate(text: string | undefined, maxLength: number): string {
  if (!text) return '-';
  if (text.length <= maxLength) return text;
  return `${text.slice(0, maxLength)}...`;
}

/**
 * Single row component for npm search results.
 */
export function NpmResultRow({ result, onSelect }: NpmResultRowProps): JSX.Element {
  const handleClick = (): void => {
    onSelect(result.id);
  };

  const handleKeyDown = (event: React.KeyboardEvent): void => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onSelect(result.id);
    }
  };

  return (
    <tr
      className="npm-result-row"
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={0}
      role="button"
      aria-label={`View details for ${result.displayName}`}
    >
      <td className="npm-result-row__name">
        <div className="npm-result-row__name-primary">{result.displayName}</div>
        {result.description && (
          <div className="npm-result-row__description">
            {truncate(result.description, 60)}
          </div>
        )}
      </td>
      <td className="npm-result-row__version">
        {result.latestVersion}
      </td>
      <td className="npm-result-row__versions-count">
        {result.versionsCount}
      </td>
      <td className="npm-result-row__author">
        {result.author || '-'}
      </td>
      <td className="npm-result-row__updated">
        {formatDate(result.lastUpdated)}
      </td>
      <td className="npm-result-row__chevron">
        <span className="npm-result-row__chevron-icon" aria-hidden="true">
          ›
        </span>
      </td>
    </tr>
  );
}

export default NpmResultRow;

