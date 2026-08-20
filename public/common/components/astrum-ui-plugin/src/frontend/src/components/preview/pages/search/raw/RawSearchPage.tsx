

import React, { useCallback } from 'react';
import {
  Box,
  Flex,
  Heading,
  Text,
  Button,
  ScrollArea,
} from '@radix-ui/themes';

import { useRawSearch } from './useRawSearch';
import { RawSearchFilters } from './RawSearchFilters';
import { RawSearchResults } from './RawSearchResults';
import type { RawResult } from './raw.types';

export interface RawSearchPageProps {
  onNavigateToDetail?: (id: string) => void;
}

/**
 * Raw format search page.
 * Allows searching for files in raw repositories.
 */
export function RawSearchPage({ onNavigateToDetail }: RawSearchPageProps): React.ReactElement {
  const { state, setFilters, search, loadMore, hasMore } = useRawSearch();

  const handleResultClick = useCallback(
    (result: RawResult) => {
      onNavigateToDetail?.(`raw:${result.id}`);
    },
    [onNavigateToDetail]
  );

  return (
    <ScrollArea scrollbars="vertical" style={{ height: '100%' }}>
      <Box p="6">
        <Flex direction="column" gap="4">
          {/* Header */}
          <Box>
            <Heading size="6" mb="1">Raw Search</Heading>
            <Text color="gray">Search for files in raw format repositories</Text>
          </Box>

          {/* Filters */}
          <RawSearchFilters
            filters={state.filters}
            onFiltersChange={setFilters}
            onSearch={search}
            loading={state.loading}
          />

          {/* Results */}
          <RawSearchResults
            results={state.results}
            loading={state.loading}
            error={state.error}
            onResultClick={handleResultClick}
            onLoadMore={loadMore}
            hasMore={hasMore}
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

export default RawSearchPage;
