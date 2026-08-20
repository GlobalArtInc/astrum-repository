

import React from 'react';
import type { HelmResult } from './helm.types';

import './HelmResultRow.scss';

export interface HelmResultRowProps {
  /** The Helm result to display */
  result: HelmResult;
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
 * Single row component for Helm search results.
 */
export function HelmResultRow({ result, onSelect }: HelmResultRowProps): JSX.Element {
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
      className="helm-result-row"
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={0}
      role="button"
      aria-label={`View details for ${result.displayName}`}
    >
      <td className="helm-result-row__name">
        <div className="helm-result-row__name-wrapper">
          {result.icon && (
            <img 
              src={result.icon} 
              alt="" 
              className="helm-result-row__icon"
              onError={(e) => {
                (e.target as HTMLImageElement).style.display = 'none';
              }}
            />
          )}
          <div className="helm-result-row__name-content">
            <div className="helm-result-row__name-primary">{result.displayName}</div>
            {result.description && (
              <div className="helm-result-row__description">
                {truncate(result.description, 60)}
              </div>
            )}
          </div>
        </div>
      </td>
      <td className="helm-result-row__version">
        {result.latestVersion}
      </td>
      <td className="helm-result-row__app-version">
        {result.appVersion || '-'}
      </td>
      <td className="helm-result-row__versions-count">
        {result.versionsCount}
      </td>
      <td className="helm-result-row__updated">
        {formatDate(result.lastUpdated)}
      </td>
      <td className="helm-result-row__chevron">
        <span className="helm-result-row__chevron-icon" aria-hidden="true">
          ›
        </span>
      </td>
    </tr>
  );
}

export default HelmResultRow;


