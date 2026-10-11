

import React from 'react';
import { Box, Flex, Text, Table, Spinner, Callout, Code } from '@radix-ui/themes';
import { Search, AlertCircle } from 'lucide-react';
import type { CustomSearchResult } from './custom.types';
import { FormatBadge } from '../../browse/shared/FormatBadge';
import plural, { pageLanguage } from '../../../../../interface/plural';
import S from '../../../constants/pages/search/CustomSearchStrings';

export interface CustomSearchResultsProps {
  results: readonly CustomSearchResult[];
  loading: boolean;
  error?: string;
  totalCount: number;
  isInitial?: boolean;
}

function formatDate(isoDate: string | undefined): string {
  if (!isoDate) {
    return '—';
  }
  const date = new Date(isoDate);
  if (Number.isNaN(date.getTime())) {
    return '—';
  }
  return date.toLocaleDateString(pageLanguage(), {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function displayName(result: CustomSearchResult): string {
  if (result.format === 'npm' && result.group) {
    return `@${result.group.replace(/^@/, '')}/${result.name}`;
  }
  return result.name;
}

/**
 * Opens a result at its first file in the browse tree of its repository, as the search page does.
 */
function openInRepository(result: CustomSearchResult): void {
  const path = result.path ? result.path.replace(/^\/+/, '') : '';
  const repository = encodeURIComponent(result.repository);
  window.location.hash = path
    ? `#browse/${repository}/${path}?tab=summary`
    : `#browse/${repository}/`;
}

export function CustomSearchResults({
  results,
  loading,
  error,
  totalCount,
  isInitial = true,
}: CustomSearchResultsProps): JSX.Element {
  // Initial state
  if (isInitial && results.length === 0 && !loading && !error) {
    return (
      <Flex justify="center" align="center" p="6">
        <Flex direction="column" align="center" gap="2">
          <Search size={48} color="var(--gray-8)" />
          <Text size="3" color="gray">{S.START_TITLE}</Text>
          <Text size="2" color="gray" align="center">{S.START_DESCRIPTION}</Text>
        </Flex>
      </Flex>
    );
  }

  // Loading state
  if (loading && results.length === 0) {
    return (
      <Flex justify="center" align="center" p="6">
        <Flex direction="column" align="center" gap="3">
          <Spinner size="3" />
          <Text color="gray">{S.SEARCHING}</Text>
        </Flex>
      </Flex>
    );
  }

  // Error state
  if (error) {
    return (
      <Callout.Root color="red">
        <Callout.Icon><AlertCircle size={16} /></Callout.Icon>
        <Callout.Text>{error}</Callout.Text>
      </Callout.Root>
    );
  }

  // Empty state
  if (results.length === 0) {
    return (
      <Flex justify="center" align="center" p="6">
        <Flex direction="column" align="center" gap="2">
          <Search size={32} color="var(--gray-8)" />
          <Text color="gray">{S.EMPTY_TITLE}</Text>
          <Text size="1" color="gray">{S.EMPTY_DESCRIPTION}</Text>
        </Flex>
      </Flex>
    );
  }

  return (
    <Box>
      <Flex justify="between" align="center" mb="3">
        <Text size="2" color="gray">
          {plural(S.FOUND, totalCount)}
        </Text>
      </Flex>

      <Box style={{ overflowX: 'auto' }}>
        <Table.Root variant="surface">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeaderCell>{S.COLUMNS.FORMAT}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{S.COLUMNS.NAME}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{S.COLUMNS.GROUP}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{S.COLUMNS.VERSION}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{S.COLUMNS.REPOSITORY}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{S.COLUMNS.LAST_MODIFIED}</Table.ColumnHeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {results.map((result) => (
              <Table.Row
                key={`${result.repository}:${result.id}`}
                tabIndex={0}
                role="button"
                aria-label={S.OPEN(displayName(result))}
                style={{ cursor: 'pointer' }}
                onClick={() => openInRepository(result)}
                onKeyDown={(event) => {
                  if (event.target === event.currentTarget && (event.key === 'Enter' || event.key === ' ')) {
                    event.preventDefault();
                    openInRepository(result);
                  }
                }}
              >
                <Table.Cell>
                  <FormatBadge format={result.format} radius="full" />
                </Table.Cell>
                <Table.Cell style={{ whiteSpace: 'nowrap' }}>
                  <Text weight="medium">{displayName(result)}</Text>
                </Table.Cell>
                <Table.Cell>
                  <Text size="2" color="gray">{(result.format !== 'npm' && result.group) || '—'}</Text>
                </Table.Cell>
                <Table.Cell style={{ whiteSpace: 'nowrap' }}>
                  {result.version ? <Code size="2">{result.version}</Code> : <Text size="2" color="gray">—</Text>}
                </Table.Cell>
                <Table.Cell>
                  <Text size="2" color="gray">{result.repository}</Text>
                </Table.Cell>
                <Table.Cell>
                  <Text size="2" color="gray" style={{ whiteSpace: 'nowrap' }}>{formatDate(result.lastModified)}</Text>
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table.Root>
      </Box>

      {loading && results.length > 0 && (
        <Flex justify="center" p="3">
          <Text size="2" color="gray">{S.LOADING_MORE}</Text>
        </Flex>
      )}
    </Box>
  );
}

export default CustomSearchResults;
