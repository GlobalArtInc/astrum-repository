

import React from 'react';
import type { GolangResult } from './golang.types';

import './GolangResultRow.scss';

export interface GolangResultRowProps {
  /** The Go module result to display */
  result: GolangResult;
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
 * Single row component for Go module search results.
 */
export function GolangResultRow({ result, onSelect }: GolangResultRowProps): JSX.Element {
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
      className="golang-result-row"
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={0}
      role="button"
      aria-label={`View details for ${result.module}`}
    >
      <td className="golang-result-row__module">
        <div className="golang-result-row__module-primary">{result.module}</div>
        {result.description && (
          <div className="golang-result-row__description">
            {truncate(result.description, 60)}
          </div>
        )}
      </td>
      <td className="golang-result-row__version">
        {result.latestVersion}
      </td>
      <td className="golang-result-row__versions-count">
        {result.versionsCount}
      </td>
      <td className="golang-result-row__license">
        {result.license || '-'}
      </td>
      <td className="golang-result-row__updated">
        {formatDate(result.lastUpdated)}
      </td>
      <td className="golang-result-row__chevron">
        <span className="golang-result-row__chevron-icon" aria-hidden="true">
          ›
        </span>
      </td>
    </tr>
  );
}

export default GolangResultRow;


