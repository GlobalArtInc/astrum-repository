

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { Box, Text, TextField } from '@radix-ui/themes';
import { Search, Layers, Plus } from 'lucide-react';

import {
  EntityTable,
  EmptyState,
  HelpSection,
  LoadingState,
  ErrorState,
  type TableColumn,
} from '../../../../shared';
import { useContentSelectorsApi } from './useContentSelectorsApi';
import { ContentSelector } from './types';

import './ContentSelectorsList.scss';
import CommonStrings from '../../../../constants/CommonStrings';
import ContentSelectorsStrings from '../../../../constants/pages/settings/repository/ContentSelectorsStrings';

interface ContentSelectorsListProps {
  onSelect: (name: string) => void;
  onCreate: () => void;
  /**
   * When false, hides the empty-state Create button so a user without
   * astrum:selectors:create cannot start a create flow (ASTRUM-54212). The header
   * Create button is gated separately in ContentSelectorsPage. Defaults to true.
   */
  canCreate?: boolean;
}

/**
 * ContentSelectorsList - Sortable, filterable table of content selectors
 * Uses shared EntityTable component for consistent UX
 */
export function ContentSelectorsList({ onSelect, onCreate, canCreate = true }: ContentSelectorsListProps) {
  const [selectors, setSelectors] = useState<ContentSelector[]>([]);
  const [filter, setFilter] = useState('');
  const [sortBy, setSortBy] = useState<string>('name');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const { fetchContentSelectors } = useContentSelectorsApi();

  // Load selectors on mount
  useEffect(() => {
    setIsLoading(true);
    setLoadError(null);
    fetchContentSelectors()
      .then(setSelectors)
      .catch((err) => setLoadError(err.message))
      .finally(() => setIsLoading(false));
  }, [fetchContentSelectors]);

  // Define table columns
  const columns: TableColumn<ContentSelector>[] = useMemo(() => [
    {
      id: 'name',
      header: CommonStrings.LABELS.NAME,
      accessor: (item) => <Text weight="medium">{item.name}</Text>,
      sortable: true,
    },
    {
      id: 'type',
      header: CommonStrings.LABELS.TYPE,
      accessor: (item) => item.type?.toUpperCase() || '',
      sortable: true,
    },
    {
      id: 'description',
      header: CommonStrings.LABELS.DESCRIPTION,
      accessor: (item) => (
        <Text className="content-selectors-list__description">
          {item.description || '—'}
        </Text>
      ),
      sortable: true,
    },
    {
      id: 'expression',
      header: ContentSelectorsStrings.LIST.EXPRESSION,
      accessor: (item) => (
        <Text className="content-selectors-list__expression">
          {item.expression || '—'}
        </Text>
      ),
      sortable: true,
    },
  ], []);

  // Handle sort changes
  const handleSort = useCallback((column: string) => {
    if (sortBy === column) {
      setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortBy(column);
      setSortDirection('asc');
    }
  }, [sortBy]);

  // Filter and sort selectors
  const filteredSelectors = useMemo(() => {
    let result = selectors;

    // Apply filter
    if (filter) {
      const lowerFilter = filter.toLowerCase();
      result = result.filter(
        (s) =>
          s.name.toLowerCase().includes(lowerFilter) ||
          (s.description?.toLowerCase().includes(lowerFilter)) ||
          (s.expression?.toLowerCase().includes(lowerFilter))
      );
    }

    // Apply sort
    if (sortBy && sortDirection) {
      result = [...result].sort((a, b) => {
        const aVal = (a[sortBy as keyof ContentSelector] || '').toString().toLowerCase();
        const bVal = (b[sortBy as keyof ContentSelector] || '').toString().toLowerCase();
        const cmp = aVal.localeCompare(bVal);
        return sortDirection === 'asc' ? cmp : -cmp;
      });
    }

    return result;
  }, [selectors, filter, sortBy, sortDirection]);

  // Retry loading on error
  const handleRetry = useCallback(() => {
    setIsLoading(true);
    setLoadError(null);
    fetchContentSelectors()
      .then(setSelectors)
      .catch((err) => setLoadError(err.message))
      .finally(() => setIsLoading(false));
  }, [fetchContentSelectors]);

  // Show loading state
  if (isLoading) {
    return (
      <Box className="content-selectors-list">
        <LoadingState message={ContentSelectorsStrings.LIST.LOADING_CONTENT_SELECTORS} />
      </Box>
    );
  }

  // Show error state
  if (loadError) {
    return (
      <Box className="content-selectors-list">
        <ErrorState
          title={ContentSelectorsStrings.LIST.FAILED_LOAD_CONTENT_SELECTORS}
          message={loadError}
          onRetry={handleRetry}
        />
      </Box>
    );
  }

  // Determine empty state based on filter
  const renderEmptyState = () => {
    if (filter) {
      // No results for filter
      return (
        <EmptyState
          icon={Search}
          title={ContentSelectorsStrings.LIST.NO_MATCHING_CONTENT_SELECTORS}
          description={ContentSelectorsStrings.LIST.NO_CONTENT_SELECTORS_MATCH_TRY(filter)}
          size="small"
        />
      );
    }

    // No selectors at all
    return (
      <EmptyState
        icon={Layers}
        title={ContentSelectorsStrings.LIST.NO_CONTENT_SELECTORS}
        description={ContentSelectorsStrings.LIST.CREATE_FIRST_CONTENT_SELECTOR_DEFINE}
        action={canCreate ? {
          label: ContentSelectorsStrings.LIST.CREATE_SELECTOR,
          onClick: onCreate,
          icon: Plus,
        } : undefined}
        secondaryAction={{
          label: ContentSelectorsStrings.LIST.LEARN_MORE_ABOUT_CONTENT_SELECTORS,
          href: 'http://links.globalart.dev/products/astrum/docs/content-selector',
        }}
      />
    );
  };

  return (
    <Box className="content-selectors-list">
      {/* Help Section */}
      <HelpSection
        title={ContentSelectorsStrings.LIST.WHAT_CONTENT_SELECTOR}
        content={ContentSelectorsStrings.LIST.CONTENT_SELECTORS_PROVIDE_MEANS_YOU}
        docLink={{
          label: CommonStrings.ACTIONS.LEARN_MORE,
          href: 'http://links.globalart.dev/products/astrum/docs/content-selector',
        }}
        className="content-selectors-list__help"
      />

      {/* Search Filter */}
      <Box className="content-selectors-list__toolbar">
        <TextField.Root
          placeholder={ContentSelectorsStrings.LIST.FILTER_NAME_DESCRIPTION_PLACEHOLDER}
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="content-selectors-list__search"
          data-analytics-id="as-content-selector-filter"
        >
          <TextField.Slot>
            <Search size={16} />
          </TextField.Slot>
        </TextField.Root>
      </Box>

      {/* Entity Table */}
      <EntityTable<ContentSelector>
        data={filteredSelectors}
        columns={columns}
        getRowKey={(item) => item.name}
        getRowTestId={(item) => `selector-row-${item.name}`}
        onRowClick={(item) => onSelect(item.name)}
        sortBy={sortBy}
        sortDirection={sortDirection}
        onSort={handleSort}
        emptyState={renderEmptyState()}
        ariaLabel={ContentSelectorsStrings.LIST.CONTENT_SELECTORS_TABLE_ARIA}
        className="content-selectors-list__table"
      />
    </Box>
  );
}

export default ContentSelectorsList;
