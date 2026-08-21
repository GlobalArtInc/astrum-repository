

import React, { useCallback, useRef, useEffect } from 'react';
import { X } from 'lucide-react';

/** Debounce delay for text filter changes (ms) */
const TEXT_FILTER_DEBOUNCE_MS = 2000; // 2 seconds for testing

export interface TextFilterProps {
  /** Filter ID */
  id: string;
  /** Display label */
  label: string;
  /** Current value */
  value: string;
  /** Callback when value changes */
  onChange: (id: string, value: string) => void;
  /** Callback to trigger search (called after debounce or immediately on Enter) */
  onSearch?: () => void;
  /** Placeholder text */
  placeholder?: string;
  /** Whether the filter is disabled */
  disabled?: boolean;
}

/**
 * Text input filter component for the search sidebar.
 * 
 * Auto-applies filter with 300ms debounce as user types.
 * Enter key triggers immediate search without waiting for debounce.
 * Escape key clears the filter.
 */
export function TextFilter({
  id,
  label,
  value,
  onChange,
  onSearch,
  placeholder,
  disabled = false,
}: TextFilterProps): JSX.Element {
  // Track debounce timer
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Clean up debounce timer on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      
      // Update UI state immediately
      onChange(id, newValue);
      
      // Cancel any pending debounced search
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      
      // Schedule debounced search
      if (onSearch) {
        debounceRef.current = setTimeout(() => {
          onSearch();
        }, TEXT_FILTER_DEBOUNCE_MS);
      }
    },
    [id, onChange, onSearch]
  );

  const handleClear = useCallback(() => {
    // Cancel any pending debounced search
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    
    onChange(id, '');
    
    // Trigger immediate search on clear
    if (onSearch) {
      // Use setTimeout(0) to ensure state update is applied first
      setTimeout(onSearch, 0);
    }
  }, [id, onChange, onSearch]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Escape' && value) {
        e.preventDefault();
        handleClear();
      } else if (e.key === 'Enter') {
        e.preventDefault();
        // Cancel pending debounce and execute immediately
        if (debounceRef.current) {
          clearTimeout(debounceRef.current);
        }
        onSearch?.();
      }
    },
    [value, handleClear, onSearch]
  );

  return (
    <div className="text-filter">
      <label className="text-filter__label" htmlFor={`filter-${id}`}>
        {label}
      </label>
      <div className="text-filter__input-wrapper">
        <input
          id={`filter-${id}`}
          type="text"
          className="text-filter__input"
          value={value}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled}
        />
        {value && !disabled && (
          <button
            type="button"
            className="text-filter__clear"
            onClick={handleClear}
            aria-label={`Clear ${label} filter`}
          >
            <X size={14} />
          </button>
        )}
      </div>
    </div>
  );
}

export default TextFilter;


