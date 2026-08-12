

import React from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Package } from 'lucide-react';
import type { SearchResultItem as SearchResult } from './browse.api';

export interface SearchResultItemProps {
  result: SearchResult;
  onClick: () => void;
  onMouseDown: () => void;
  isSelected?: boolean;
}

/**
 * Individual search result item displayed in the dropdown.
 */
export function SearchResultItem({
  result,
  onClick,
  onMouseDown,
  isSelected = false,
}: SearchResultItemProps): JSX.Element {
  const displayName = result.group
    ? `${result.group}:${result.name}`
    : result.name;

  const displayVersion = result.version || '';

  return (
    <Box
      className={`in-repo-search__result-item${isSelected ? ' in-repo-search__result-item--selected' : ''}`}
      onClick={onClick}
      onMouseDown={onMouseDown}
      role="option"
      aria-selected={isSelected}
      tabIndex={-1}
      p="2"
      style={{
        cursor: 'pointer',
        borderBottom: '1px solid var(--gray-4)',
      }}
    >
      <Flex align="center" gap="2">
        <Package size={16} className="in-repo-search__result-icon" aria-hidden />
        <Box style={{ flex: 1, minWidth: 0 }}>
          <Text
            size="2"
            weight="medium"
            className="in-repo-search__result-name"
            style={{
              display: 'block',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {displayName}
          </Text>
          {displayVersion && (
            <Text
              size="1"
              color="gray"
              className="in-repo-search__result-version"
              style={{ display: 'block' }}
            >
              {displayVersion}
            </Text>
          )}
        </Box>
      </Flex>
    </Box>
  );
}

export default SearchResultItem;
