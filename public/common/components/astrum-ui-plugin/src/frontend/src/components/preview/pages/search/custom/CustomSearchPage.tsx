import React, { useState } from 'react';
import {
  Box,
  Flex,
  Heading,
  Text,
  Button,
  ScrollArea,
} from '@radix-ui/themes';

import { useCustomSearch } from './useCustomSearch';
import { CustomSearchBuilder } from './CustomSearchBuilder';
import { CustomSearchResults } from './CustomSearchResults';
import S from '../../../constants/pages/search/CustomSearchStrings';
import '../../../shared/PageScrollArea.scss';

export interface CustomSearchPageProps {
  /** Optional callback when navigating to component detail */
  onNavigateToDetail?: (componentId: string) => void;
}

/**
 * Custom Search Page component.
 *
 * Allows users to build custom search queries using dynamic filters.
 * Supports multiple filter criteria with different fields and operators.
 *
 * Features:
 * - Dynamic filter builder (add/remove filter rows)
 * - Multiple field types (format, repository, group, name, version, keyword, and tag in the PRO edition)
 * - A value matched whole or by its start
 * - Results display across all formats, each opening in its repository
 * - Pagination support
 */
export function CustomSearchPage({
  onNavigateToDetail: _onNavigateToDetail,
}: CustomSearchPageProps): JSX.Element {
  // Track whether a search has been performed
  const [hasSearched, setHasSearched] = useState(false);

  const {
    state,
    addFilter,
    removeFilter,
    updateFilter,
    search,
    loadMore,
    clear,
    hasMore,
    hasFilters,
  } = useCustomSearch();

  /**
   * Handle search execution.
   */
  const handleSearch = async (): Promise<void> => {
    setHasSearched(true);
    await search();
  };

  /**
   * Handle clear all.
   */
  const handleClear = (): void => {
    setHasSearched(false);
    clear();
  };

  return (
    <ScrollArea scrollbars="vertical" style={{ height: '100%' }} className="page-scroll-area">
      <Box px={{ initial: '4', md: '6' }} py={{ initial: '4', md: '5' }}>
        <Flex direction="column" gap="5">
          {/* Header */}
          <Flex direction="column" gap="1">
            <Heading as="h1" size="7" weight="bold">{S.TITLE}</Heading>
            <Text size="2" color="gray">{S.SUBTITLE}</Text>
          </Flex>

          {/* Search Builder */}
          <CustomSearchBuilder
            filters={state.filters}
            onUpdateFilter={updateFilter}
            onRemoveFilter={removeFilter}
            onAddFilter={addFilter}
            onSearch={handleSearch}
            onClear={handleClear}
            loading={state.loading}
            hasFilters={hasFilters}
          />

          {/* Results */}
          <CustomSearchResults
            results={state.results}
            loading={state.loading}
            error={state.error}
            totalCount={state.totalCount}
            isInitial={!hasSearched}
          />

          {/* Load More */}
          {hasMore && !state.loading && (
            <Flex justify="center">
              <Button variant="soft" onClick={loadMore}>
                {S.LOAD_MORE}
              </Button>
            </Flex>
          )}
        </Flex>
      </Box>
    </ScrollArea>
  );
}

export default CustomSearchPage;
