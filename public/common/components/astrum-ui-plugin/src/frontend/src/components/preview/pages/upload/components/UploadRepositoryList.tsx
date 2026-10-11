

import React, { useCallback } from 'react';
import { Box, Flex, Table, Text, Spinner } from '@radix-ui/themes';
import { ChevronUp, ChevronDown, ChevronRight, AlertCircle } from 'lucide-react';

import { CopyUrlButton } from '../../browse/actions/CopyUrlButton';
import { FormatBadge } from '../../../shared';
import type { UploadRepositoryListProps, SortColumn } from '../upload.types';
import { UPLOAD_STRINGS } from '../upload.types';
import UploadStrings from '../../../constants/pages/upload/UploadStrings';


/**
 * Renders the sort indicator icon based on current sort state.
 */
function SortIndicator({
  column,
  currentColumn,
  direction,
}: {
  column: SortColumn;
  currentColumn: SortColumn | null;
  direction: 'asc' | 'desc' | null;
}): JSX.Element | null {
  if (currentColumn !== column || !direction) {
    return null;
  }

  return direction === 'asc' ? (
    <ChevronUp size={14} className="upload-repository-list__sort-icon" />
  ) : (
    <ChevronDown size={14} className="upload-repository-list__sort-icon" />
  );
}

/**
 * UploadRepositoryList displays a table of repositories that support file uploads.
 *
 * Features:
 * - Sortable columns (Name, Format)
 * - Copy URL button for each repository
 * - Click row to navigate to upload form
 * - Loading and error states
 * - Empty state message
 */
export function UploadRepositoryList({
  repositories,
  loading,
  error,
  sortColumn,
  sortDirection,
  onSort,
  onSelect,
}: UploadRepositoryListProps): JSX.Element {
  const handleRowClick = useCallback(
    (repoName: string) => {
      onSelect(repoName);
    },
    [onSelect]
  );

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent, repoName: string) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        onSelect(repoName);
      }
    },
    [onSelect]
  );

  const getSortAriaLabel = (column: SortColumn): string => {
    if (sortColumn !== column) {
      return UploadStrings.REPOSITORY_LIST.SORT_BY(UPLOAD_STRINGS.columns[column]);
    }
    if (sortDirection === 'asc') {
      return UploadStrings.REPOSITORY_LIST.SORTED_ASCENDING_CLICK_SORT_DESCENDING(UPLOAD_STRINGS.columns[column]);
    }
    if (sortDirection === 'desc') {
      return UploadStrings.REPOSITORY_LIST.SORTED_DESCENDING_CLICK_REMOVE_SORT(UPLOAD_STRINGS.columns[column]);
    }
    return UploadStrings.REPOSITORY_LIST.SORT_BY(UPLOAD_STRINGS.columns[column]);
  };

  // Loading state
  if (loading) {
    return (
      <Box className="upload-repository-list upload-repository-list--loading" p="6">
        <Flex align="center" justify="center" gap="3">
          <Spinner size="3" />
          <Text color="gray">{UPLOAD_STRINGS.loadingMessage}</Text>
        </Flex>
      </Box>
    );
  }

  // Error state
  if (error) {
    return (
      <Box className="upload-repository-list upload-repository-list--error" p="6">
        <Flex align="center" justify="center" gap="2">
          <AlertCircle size={20} className="upload-repository-list__error-icon" />
          <Text color="red">
            {UPLOAD_STRINGS.errorPrefix} {error}
          </Text>
        </Flex>
      </Box>
    );
  }

  // Empty state
  if (repositories.length === 0) {
    return (
      <Box className="upload-repository-list upload-repository-list--empty" p="6">
        <Text color="gray" align="center">
          {UPLOAD_STRINGS.emptyMessage}
        </Text>
      </Box>
    );
  }

  return (
    <Box className="upload-repository-list">
      <Table.Root variant="surface" size="3" className="upload-repository-list__table">
        <Table.Header>
          <Table.Row>
            {/* Name Column - Sortable */}
            <Table.ColumnHeaderCell
              className="upload-repository-list__header upload-repository-list__header--sortable"
              onClick={() => onSort('name')}
              aria-label={getSortAriaLabel('name')}
            >
              <Flex align="center" gap="1">
                {UPLOAD_STRINGS.columns.name}
                <SortIndicator
                  column="name"
                  currentColumn={sortColumn}
                  direction={sortDirection}
                />
              </Flex>
            </Table.ColumnHeaderCell>

            {/* Format Column - Sortable */}
            <Table.ColumnHeaderCell
              className="upload-repository-list__header upload-repository-list__header--sortable"
              onClick={() => onSort('format')}
              aria-label={getSortAriaLabel('format')}
            >
              <Flex align="center" gap="1">
                {UPLOAD_STRINGS.columns.format}
                <SortIndicator
                  column="format"
                  currentColumn={sortColumn}
                  direction={sortDirection}
                />
              </Flex>
            </Table.ColumnHeaderCell>

            {/* URL Column - Not sortable */}
            <Table.ColumnHeaderCell className="upload-repository-list__header upload-repository-list__header--url">
              {UPLOAD_STRINGS.columns.url}
            </Table.ColumnHeaderCell>

            {/* Chevron Column */}
            <Table.ColumnHeaderCell className="upload-repository-list__header upload-repository-list__header--chevron" />
          </Table.Row>
        </Table.Header>

        <Table.Body>
          {repositories.map((repo) => (
            <Table.Row
              key={repo.name}
              className="upload-repository-list__row"
              onClick={() => handleRowClick(repo.name)}
              onKeyDown={(e) => handleKeyDown(e, repo.name)}
              tabIndex={0}
              role="button"
              aria-label={UploadStrings.REPOSITORY_LIST.UPLOAD_TO(repo.name)}
            >
              <Table.Cell className="upload-repository-list__cell upload-repository-list__cell--name">
                <Text weight="medium">{repo.name}</Text>
              </Table.Cell>

              <Table.Cell className="upload-repository-list__cell upload-repository-list__cell--format">
                <FormatBadge format={repo.format} />
              </Table.Cell>

              <Table.Cell className="upload-repository-list__cell upload-repository-list__cell--url">
                <CopyUrlButton
                  url={repo.url}
                  tooltipText={UPLOAD_STRINGS.copyUrlTooltip}
                  successMessage={UPLOAD_STRINGS.urlCopied}
                  size="small"
                />
              </Table.Cell>

              <Table.Cell className="upload-repository-list__cell upload-repository-list__cell--chevron">
                <ChevronRight size={16} className="upload-repository-list__chevron" />
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table.Root>
    </Box>
  );
}

export default UploadRepositoryList;

