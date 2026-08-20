

import React from 'react';
import {
  Box,
  Flex,
  TextField,
  Button,
  Text,
} from '@radix-ui/themes';
import { X } from 'lucide-react';
import type { NpmSearchFilters as FilterValues } from './npm.types';

export interface NpmSearchFiltersProps {
  /** Current filter values */
  values: FilterValues;
  /** Callback when filter values change */
  onChange: (values: FilterValues) => void;
  /** Callback when search is triggered */
  onSearch: () => void;
  /** Callback to clear all filters */
  onClear: () => void;
  /** Whether a search is in progress */
  loading?: boolean;
}

/**
 * Filter controls for npm package search.
 */
export function NpmSearchFilters({
  values,
  onChange,
  onSearch,
  onClear,
  loading = false,
}: NpmSearchFiltersProps): JSX.Element {
  const handleKeyDown = (event: React.KeyboardEvent): void => {
    if (event.key === 'Enter') {
      onSearch();
    }
  };

  const hasFilters = values.scope || values.version || values.tag;

  return (
    <Box p="4" style={{ backgroundColor: 'var(--gray-2)', borderRadius: 'var(--radius-3)' }}>
      <Flex direction="column" gap="3">
        <Flex gap="3" wrap="wrap">
          {/* Scope */}
          <Box style={{ flex: '1 1 150px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Scope
            </Text>
            <TextField.Root
              placeholder="e.g., @angular"
              value={values.scope ?? ''}
              onChange={(e) => onChange({ ...values, scope: e.target.value || undefined })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>

          {/* Version */}
          <Box style={{ flex: '1 1 120px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Version
            </Text>
            <TextField.Root
              placeholder="e.g., 18.2.0"
              value={values.version ?? ''}
              onChange={(e) => onChange({ ...values, version: e.target.value || undefined })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>

          {/* Tag */}
          <Box style={{ flex: '1 1 120px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Tag
            </Text>
            <TextField.Root
              placeholder="e.g., latest"
              value={values.tag ?? ''}
              onChange={(e) => onChange({ ...values, tag: e.target.value || undefined })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>
        </Flex>

        {/* Actions */}
        {hasFilters && (
          <Flex justify="end">
            <Button variant="ghost" onClick={onClear} disabled={loading}>
              <X size={14} />
              Clear Filters
            </Button>
          </Flex>
        )}
      </Flex>
    </Box>
  );
}

export default NpmSearchFilters;
