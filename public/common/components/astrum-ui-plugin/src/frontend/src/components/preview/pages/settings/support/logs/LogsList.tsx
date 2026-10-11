

import React, { useCallback } from 'react';
import { Box, Flex, Text, Table } from '@radix-ui/themes';
import { Search, Pencil, ArrowUp, ArrowDown, FileText, Loader2 } from 'lucide-react';
import { HumanReadableUtils } from '../../../../../../interface/HumanReadableUtils';
import { DateTime } from 'luxon';
import { pageLanguage } from '../../../../../../interface/plural';

import { SettingsTextInput, SettingsAlert } from '../../../../shared/form';
import { useLogsList } from './useLogsList';
import { LogSortField } from './types';

import './LogsList.scss';
import LogsStrings from '../../../../constants/pages/settings/support/LogsStrings';

interface LogsListProps {
  onSelect: (filename: string) => void;
}

/**
 * LogsList - Displays a searchable, sortable list of log files
 */
export function LogsList({ onSelect }: LogsListProps) {
  const {
    filteredLogs,
    filter,
    sortField,
    sortDirection,
    error,
    isLoading,
    setFilter,
    handleSort,
  } = useLogsList();

  const renderSortIcon = useCallback((field: LogSortField) => {
    if (sortField !== field) return null;
    return sortDirection === 'asc' ? (
      <ArrowUp size={14} className="logs-list__sort-icon" />
    ) : (
      <ArrowDown size={14} className="logs-list__sort-icon" />
    );
  }, [sortField, sortDirection]);

  const formatSize = (bytes: number): string => {
    return HumanReadableUtils.bytesToString(bytes);
  };

  const formatDate = (timestamp: number): string => {
    return DateTime.fromMillis(timestamp).toLocaleString(DateTime.DATETIME_SHORT_WITH_SECONDS, { locale: pageLanguage() });
  };

  if (isLoading) {
    return (
      <Box className="logs-list logs-list--loading">
        <Flex align="center" justify="center" gap="2" py="9">
          <Loader2 size={20} className="logs-list__spinner" />
          <Text size="2">{LogsStrings.LIST.LOADING_LOG_FILES}</Text>
        </Flex>
      </Box>
    );
  }

  return (
    <Box className="logs-list">
      {/* Filter */}
      <Box className="logs-list__toolbar">
        <SettingsTextInput
          value={filter}
          onChange={setFilter}
          placeholder={LogsStrings.LIST.FILTER_FILE_NAME_PLACEHOLDER}
          className="logs-list__filter"
          icon={<Search size={16} />}
        />
      </Box>

      {/* Error alert */}
      {error && (
        <Box mb="3">
          <SettingsAlert type="error">
            {error}
          </SettingsAlert>
        </Box>
      )}

      {/* Table */}
      <Table.Root className="logs-list__table">
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeaderCell
              className="logs-list__header-cell logs-list__header-cell--sortable"
              onClick={() => handleSort('fileName')}
            >
              <Flex align="center" gap="1">
                {LogsStrings.LIST.FILE_NAME}
                {renderSortIcon('fileName')}
              </Flex>
            </Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell
              className="logs-list__header-cell logs-list__header-cell--sortable logs-list__header-cell--size"
              onClick={() => handleSort('size')}
            >
              <Flex align="center" gap="1">
                {LogsStrings.LIST.SIZE}
                {renderSortIcon('size')}
              </Flex>
            </Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell
              className="logs-list__header-cell logs-list__header-cell--sortable logs-list__header-cell--modified"
              onClick={() => handleSort('lastModified')}
            >
              <Flex align="center" gap="1">
                {LogsStrings.LIST.LAST_MODIFIED}
                {renderSortIcon('lastModified')}
              </Flex>
            </Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell className="logs-list__header-cell logs-list__header-cell--action" />
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {filteredLogs.length === 0 ? (
            <Table.Row>
              <Table.Cell colSpan={4} className="logs-list__empty-cell">
                <Flex align="center" justify="center" py="6">
                  <Text size="2" color="gray">
                    {filter ? LogsStrings.LIST.NO_LOG_FILES_MATCH_FILTER : LogsStrings.LIST.NO_LOG_FILES_FOUND}
                  </Text>
                </Flex>
              </Table.Cell>
            </Table.Row>
          ) : (
            filteredLogs.map((log) => (
              <Table.Row
                key={log.fileName}
                className="logs-list__row"
                onClick={() => onSelect(log.fileName)}
              >
                <Table.Cell className="logs-list__cell">
                  <Flex align="center" gap="2">
                    <FileText size={16} className="logs-list__file-icon" />
                    <Text>{log.fileName}</Text>
                  </Flex>
                </Table.Cell>
                <Table.Cell className="logs-list__cell logs-list__cell--size">
                  {formatSize(log.size)}
                </Table.Cell>
                <Table.Cell className="logs-list__cell logs-list__cell--modified">
                  {formatDate(log.lastModified)}
                </Table.Cell>
                <Table.Cell className="logs-list__cell logs-list__cell--action">
                  <Pencil size={16} className="logs-list__edit-icon" />
                </Table.Cell>
              </Table.Row>
            ))
          )}
        </Table.Body>
      </Table.Root>
    </Box>
  );
}

export default LogsList;
