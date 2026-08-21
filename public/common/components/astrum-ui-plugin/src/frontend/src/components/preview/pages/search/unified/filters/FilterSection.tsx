

import React, { useState, ReactNode } from 'react';
import { ChevronDown, ChevronRight } from 'lucide-react';

export interface FilterSectionProps {
  /** Section title */
  title: string;
  /** Whether the section is initially expanded */
  defaultExpanded?: boolean;
  /** Number badge to show (e.g., active filter count) */
  badge?: number;
  /** Child filter components */
  children: ReactNode;
}

/**
 * Collapsible filter section wrapper.
 * Used in the search sidebar to group filters.
 */
export function FilterSection({
  title,
  defaultExpanded = true,
  badge,
  children,
}: FilterSectionProps): JSX.Element {
  const [expanded, setExpanded] = useState(defaultExpanded);

  return (
    <div className="filter-section">
      <button
        type="button"
        className="filter-section__header"
        onClick={() => setExpanded(!expanded)}
        aria-expanded={expanded}
      >
        <span className="filter-section__icon">
          {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        </span>
        <span className="filter-section__title">{title}</span>
        {badge !== undefined && badge > 0 && (
          <span className="filter-section__badge">{badge}</span>
        )}
      </button>

      {expanded && (
        <div className="filter-section__content">
          {children}
        </div>
      )}
    </div>
  );
}

export default FilterSection;


