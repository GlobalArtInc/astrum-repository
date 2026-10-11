

import React, { useState, useCallback, useEffect } from 'react';
import {
  Box,
  Flex,
  Heading,
  Text,
  TextField,
  Button,
  Badge,
  ScrollArea,
} from '@radix-ui/themes';
import { Search } from 'lucide-react';

import type { YumSearchFilters as FilterValues } from './yum.types';
import { useYumSearch } from './useYumSearch';
import { YumSearchFilters } from './YumSearchFilters';
import { YumSearchResults } from './YumSearchResults';

export interface YumSearchPageProps {
  /** Callback when navigating to detail page */
  onNavigateToDetail?: (id: string) => void;
  /** Initial name filter from URL */
  initialName?: string;
  /** Initial architecture filter from URL */
  initialArchitecture?: string;
}

/**
 * Main Yum/RPM Search page component.
 *
 * Features:
 * - Search by name, version, architecture
 * - Results table showing package info
 * - Click row to navigate to detail page
 * - URL-driven state (bookmarkable)
 */
export function YumSearchPage({
  onNavigateToDetail,
  initialName = '',
  initialArchitecture = '',
}: YumSearchPageProps): JSX.Element {
  const { state, search, loadMore, clear, hasMore } = useYumSearch();

  // Search input state
  const [searchInput, setSearchInput] = useState(initialName);

  // Local filter state
  const [filters, setFilters] = useState<FilterValues>({
    name: initialName,
    version: '',
    architecture: initialArchitecture,
  });

  // Perform initial search if we have initial values
  useEffect(() => {
    if (initialName || initialArchitecture) {
      search({
        name: initialName,
        architecture: initialArchitecture,
      });
    }
  }, [initialArchitecture, initialName, search]); // Only on mount

  /**
   * Handle search submission.
   */
  const handleSearch = useCallback((): void => {
    search({
      ...filters,
      name: searchInput || filters.name,
    });
  }, [search, filters, searchInput]);

  /**
   * Handle search input keypress.
   */
  const handleSearchKeyDown = (e: React.KeyboardEvent): void => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  /**
   * Handle selecting a result to view details.
   */
  const handleSelect = useCallback((id: string): void => {
    if (onNavigateToDetail) {
      onNavigateToDetail(id);
    } else {
      // Default: navigate using hash
      window.location.hash = `#browse/search/yum/detail/${encodeURIComponent(id)}`;
    }
  }, [onNavigateToDetail]);

  /**
   * Handle clearing all filters.
   */
  const handleClear = useCallback((): void => {
    setSearchInput('');
    setFilters({
      name: '',
      version: '',
      architecture: '',
    });
    clear();
  }, [clear]);

  return (
    <ScrollArea scrollbars="vertical" style={{ height: '100%' }}>
      <Box p="6">
        <Flex direction="column" gap="4">
          {/* Header */}
          <Box>
            <Heading size="6" mb="1">Yum/RPM Search</Heading>
            <Text color="gray">Search for RPM packages by name, version, or architecture</Text>
          </Box>

          {/* Search Input */}
          <Flex gap="2" align="center">
            <Badge color="red" size="2" style={{ flexShrink: 0 }}>RPM</Badge>
            <Box style={{ flex: 1 }}>
              <TextField.Root
                size="3"
                placeholder="Search RPM packages..."
                value={searchInput}
                onChange={(e) => setSearchInput(e.target.value)}
                onKeyDown={handleSearchKeyDown}
                disabled={state.loading}
              >
                <TextField.Slot>
                  <Search size={16} />
                </TextField.Slot>
              </TextField.Root>
            </Box>
            <Button size="3" onClick={handleSearch} disabled={state.loading}>
              Search
            </Button>
          </Flex>

          {/* Filters */}
          <YumSearchFilters
            values={filters}
            onChange={setFilters}
            onSearch={handleSearch}
            onClear={handleClear}
            loading={state.loading}
          />

          {/* Results */}
          <YumSearchResults
            results={state.results}
            loading={state.loading}
            error={state.error}
            totalCount={state.totalCount}
            onSelect={handleSelect}
          />

          {/* Load More */}
          {hasMore && !state.loading && (
            <Flex justify="center">
              <Button variant="soft" onClick={loadMore}>
                Load More
              </Button>
            </Flex>
          )}
        </Flex>
      </Box>
    </ScrollArea>
  );
}

export default YumSearchPage;
