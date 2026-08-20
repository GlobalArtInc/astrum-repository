

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

import type { NpmSearchFilters as FilterValues } from './npm.types';
import { useNpmSearch } from './useNpmSearch';
import { NpmSearchFilters } from './NpmSearchFilters';
import { NpmSearchResults } from './NpmSearchResults';

export interface NpmSearchPageProps {
  /** Callback when navigating to detail page */
  onNavigateToDetail?: (id: string) => void;
  /** Initial scope filter from URL */
  initialScope?: string;
  /** Initial name filter from URL */
  initialName?: string;
}

/**
 * Main npm Search page component.
 *
 * Features:
 * - Search by scope, name, version, tag
 * - Results table showing package info
 * - Click row to navigate to detail page
 * - URL-driven state (bookmarkable)
 */
export function NpmSearchPage({
  onNavigateToDetail,
  initialScope = '',
  initialName = '',
}: NpmSearchPageProps): JSX.Element {
  const { state, search, loadMore, clear, hasMore } = useNpmSearch();

  // Search input state
  const [searchInput, setSearchInput] = useState(initialName);

  // Local filter state
  const [filters, setFilters] = useState<FilterValues>({
    scope: initialScope,
    name: initialName,
    version: '',
    tag: '',
  });

  // Perform initial search if we have initial values
  useEffect(() => {
    if (initialScope || initialName) {
      search({
        scope: initialScope,
        name: initialName,
      });
    }
  }, [initialName, initialScope, search]); // Only on mount

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
      window.location.hash = `#preview/browse/search/npm/detail/${encodeURIComponent(id)}`;
    }
  }, [onNavigateToDetail]);

  /**
   * Handle clearing all filters.
   */
  const handleClear = useCallback((): void => {
    setSearchInput('');
    setFilters({
      scope: '',
      name: '',
      version: '',
      tag: '',
    });
    clear();
  }, [clear]);

  return (
    <ScrollArea scrollbars="vertical" style={{ height: '100%' }}>
      <Box p="6">
        <Flex direction="column" gap="4">
          {/* Header */}
          <Box>
            <Heading size="6" mb="1">npm Search</Heading>
            <Text color="gray">Search for npm packages by scope, name, version, or tag</Text>
          </Box>

          {/* Search Input */}
          <Flex gap="2" align="center">
            <Badge color="red" size="2" style={{ flexShrink: 0 }}>npm</Badge>
            <Box style={{ flex: 1 }}>
              <TextField.Root
                size="3"
                placeholder="Search npm packages..."
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
          <NpmSearchFilters
            values={filters}
            onChange={setFilters}
            onSearch={handleSearch}
            onClear={handleClear}
            loading={state.loading}
          />

          {/* Results */}
          <NpmSearchResults
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

export default NpmSearchPage;


