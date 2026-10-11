

import React from 'react';
import { Card, Flex, Text, Button } from '@radix-ui/themes';
import { Plus, Search, X } from 'lucide-react';
import type { CustomFilter } from './custom.types';
import { CustomFilterRow } from './CustomFilterRow';
import { ExtJS } from '../../../../../interface/ExtJS';
import S from '../../../constants/pages/search/CustomSearchStrings';

export interface CustomSearchBuilderProps {
  filters: readonly CustomFilter[];
  onUpdateFilter: (id: string, updates: Partial<CustomFilter>) => void;
  onRemoveFilter: (id: string) => void;
  onAddFilter: () => void;
  onSearch: () => void;
  onClear: () => void;
  loading?: boolean;
  hasFilters?: boolean;
}

export function CustomSearchBuilder({
  filters,
  onUpdateFilter,
  onRemoveFilter,
  onAddFilter,
  onSearch,
  onClear,
  loading = false,
  hasFilters = false,
}: CustomSearchBuilderProps): JSX.Element {
  const proEdition = ExtJS.isProEdition();

  const handleKeyDown = (e: React.KeyboardEvent): void => {
    if (e.key === 'Enter' && !loading) {
      onSearch();
    }
  };

  return (
    <Card size="3" onKeyDown={handleKeyDown}>
      <Flex direction="column" gap="4">
        <Flex direction="column" gap="1">
          <Text as="div" size="3" weight="bold">{S.CRITERIA}</Text>
          <Text as="div" size="2" color="gray">
            {S.CRITERIA_DESCRIPTION}
          </Text>
        </Flex>

        <Flex direction="column" gap="2">
          {filters.map((filter) => (
            <CustomFilterRow
              key={filter.id}
              filter={filter}
              onUpdate={onUpdateFilter}
              onRemove={onRemoveFilter}
              isOnlyFilter={filters.length === 1}
              disabled={loading}
              proEdition={proEdition}
            />
          ))}
        </Flex>

        <Flex justify="start">
          <Button variant="ghost" onClick={onAddFilter} disabled={loading}>
            <Plus size={16} />
            {S.ADD_FILTER}
          </Button>
        </Flex>

        <Flex justify="end" gap="2">
          {hasFilters && (
            <Button variant="ghost" onClick={onClear} disabled={loading}>
              <X size={14} />
              {S.CLEAR_ALL}
            </Button>
          )}
          <Button onClick={onSearch} disabled={loading}>
            <Search size={16} />
            {loading ? S.SEARCHING : S.SEARCH}
          </Button>
        </Flex>
      </Flex>
    </Card>
  );
}

export default CustomSearchBuilder;
