

import React, { useCallback } from 'react';
import {
  Box,
  Flex,
  TextField,
  Button,
  Text,
} from '@radix-ui/themes';
import { Search } from 'lucide-react';
import type { RawSearchFilters as FilterValues } from './raw.types';

export interface RawSearchFiltersProps {
  /** Current filter values */
  filters: FilterValues;
  /** Callback when filters change */
  onFiltersChange: (filters: Partial<FilterValues>) => void;
  /** Callback when search is triggered */
  onSearch: () => void;
  /** Whether search is in progress */
  loading?: boolean;
}

/**
 * Filter controls for raw search.
 */
export function RawSearchFilters({
  filters,
  onFiltersChange,
  onSearch,
  loading = false,
}: RawSearchFiltersProps): React.ReactElement {
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && !loading) {
        onSearch();
      }
    },
    [onSearch, loading]
  );

  return (
    <Box p="4" style={{ backgroundColor: 'var(--gray-2)', borderRadius: 'var(--radius-3)' }}>
      <Flex direction="column" gap="3">
        <Flex gap="3" wrap="wrap">
          <Box style={{ flex: '1 1 200px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Keyword
            </Text>
            <TextField.Root
              placeholder="Search across all fields..."
              value={filters.keyword ?? ''}
              onChange={(e) => onFiltersChange({ keyword: e.target.value })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            >
              <TextField.Slot>
                <Search size={14} />
              </TextField.Slot>
            </TextField.Root>
          </Box>

          <Box style={{ flex: '1 1 180px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              File Name
            </Text>
            <TextField.Root
              placeholder="e.g., readme.md"
              value={filters.name ?? ''}
              onChange={(e) => onFiltersChange({ name: e.target.value })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>

          <Box style={{ flex: '1 1 150px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Path/Group
            </Text>
            <TextField.Root
              placeholder="e.g., /docs"
              value={filters.group ?? ''}
              onChange={(e) => onFiltersChange({ group: e.target.value })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>

          <Box style={{ flex: '1 1 150px' }}>
            <Text as="label" size="1" color="gray" mb="1">
              Repository
            </Text>
            <TextField.Root
              placeholder="e.g., raw-hosted"
              value={filters.repository ?? ''}
              onChange={(e) => onFiltersChange({ repository: e.target.value })}
              onKeyDown={handleKeyDown}
              disabled={loading}
            />
          </Box>
        </Flex>

        <Flex justify="end">
          <Button onClick={onSearch} disabled={loading}>
            <Search size={14} />
            {loading ? 'Searching...' : 'Search'}
          </Button>
        </Flex>
      </Flex>
    </Box>
  );
}

export default RawSearchFilters;
