

import React from 'react';
import {
  Box,
  Flex,
  TextField,
  Button,
  Text,
} from '@radix-ui/themes';
import { X } from 'lucide-react';

export interface FilterValues {
  imageName: string;
  tag: string;
  digest: string;
}

export interface DockerSearchFiltersProps {
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
 * Filter controls for Docker search.
 * Provides inputs for imageName, tag, and digest filtering.
 */
export function DockerSearchFilters({
  values,
  onChange,
  onSearch,
  onClear,
  loading = false,
}: DockerSearchFiltersProps): JSX.Element {
  const handleKeyDown = (event: React.KeyboardEvent): void => {
    if (event.key === 'Enter') {
      onSearch();
    }
  };

  const hasFilters = values.imageName || values.tag || values.digest;

  return (
    <Box p="4" style={{ backgroundColor: 'var(--gray-2)', borderRadius: 'var(--radius-3)' }}>
      <Flex direction="column" gap="3">
        <Flex gap="3" wrap="wrap">
          <Box style={{ flex: '1 1 200px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Image Name
            </Text>
            <TextField.Root
              placeholder="e.g., nginx, ubuntu"
              value={values.imageName}
              onChange={(e) => onChange({ ...values, imageName: e.target.value })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>

          <Box style={{ flex: '1 1 150px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Tag
            </Text>
            <TextField.Root
              placeholder="e.g., latest, 1.0.0"
              value={values.tag}
              onChange={(e) => onChange({ ...values, tag: e.target.value })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>

          <Box style={{ flex: '1 1 250px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Digest
            </Text>
            <TextField.Root
              placeholder="e.g., sha256:abc123..."
              value={values.digest}
              onChange={(e) => onChange({ ...values, digest: e.target.value })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>
        </Flex>

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

export default DockerSearchFilters;
