

import React, { useMemo } from 'react';
import { Box, Flex, Text, Button, Select, Checkbox, TextField } from '@radix-ui/themes';
import { RefreshCw } from 'lucide-react';

import type { AuditFilters, AuditCategory } from '@globalart/astrum-ui-plugin';
import { CATEGORY_LABELS, COMMON_EVENT_TYPES } from '@globalart/astrum-ui-plugin';
import type { Repository } from '../settings/repository/repositories/types';
import './AuditFilterSidebar.scss';
import AuditStrings from '../../constants/pages/audit/AuditStrings';
import CommonStrings from '../../constants/CommonStrings';
import { AUDIT_EVENT_STRINGS } from '../../../../utils/audit/auditStrings';

const CATEGORY_OPTIONS: Array<{ value: AuditCategory; label: string }> = [
  { value: 'security', label: CATEGORY_LABELS.security },
  { value: 'repository', label: CATEGORY_LABELS.repository },
  { value: 'configuration', label: CATEGORY_LABELS.configuration },
  { value: 'protection', label: CATEGORY_LABELS.protection },
];

const DATE_RANGE_OPTIONS = [
  { value: 'last-24-hours', label: AuditStrings.FILTERS.LAST_24_HOURS },
  { value: 'last-7-days', label: AuditStrings.FILTERS.LAST_7_DAYS },
  { value: 'last-30-days', label: AuditStrings.FILTERS.LAST_30_DAYS },
  { value: 'last-90-days', label: AuditStrings.FILTERS.LAST_90_DAYS },
];

const EVENT_TYPE_OPTIONS = [
  ...COMMON_EVENT_TYPES.map((type) => ({
    value: type,
    label: AUDIT_EVENT_STRINGS.EVENT_LABELS[type],
  })),
  { value: 'automatic-malware-removed', label: AuditStrings.FILTERS.MALWARE_CLEANED },
];

export interface AuditFilterSidebarProps {
  filters: AuditFilters;
  repositories: Repository[];
  onCategoryToggle: (category: AuditCategory) => void;
  onEventTypeToggle: (eventType: string) => void;
  onInitiatorChange: (initiator: string) => void;
  onRepositoryNameChange: (repositoryName: string) => void;
  onRepositoryTypeChange: (repositoryType: string) => void;
  onDateRangeChange: (range: AuditFilters['dateRange']) => void;
  onClearAllFilters: () => void;
  disabled?: boolean;
}

export function AuditFilterSidebar({
  filters,
  repositories,
  onCategoryToggle,
  onEventTypeToggle,
  onInitiatorChange,
  onRepositoryNameChange,
  onRepositoryTypeChange,
  onDateRangeChange,
  onClearAllFilters,
  disabled = false,
}: AuditFilterSidebarProps): JSX.Element {
  const formatOptions = useMemo(() => {
    const formats = new Set<string>();
    repositories.forEach((repo) => {
      if (repo.format) formats.add(repo.format);
    });
    return Array.from(formats)
      .sort()
      .map((f) => ({ value: f, label: f.toUpperCase() }));
  }, [repositories]);

  const repoOptions = useMemo(() => {
    return repositories
      .slice()
      .sort((a, b) => a.name.localeCompare(b.name))
      .map((repo) => ({ value: repo.name, label: repo.name }));
  }, [repositories]);

  return (
    <Box p="1" pt="0">
      <Button
        variant="outline"
        color="gray"
        size="2"
        mb="4"
        onClick={onClearAllFilters}
        disabled={disabled}
      >
        <RefreshCw size={12} />
        {AuditStrings.FILTERS.RESET_FILTERS}
      </Button>

      <Flex direction="column" gap="4">
        {/* Date Range */}
        <Box>
          <Text as="p" size="2" weight="bold" mb="2">
            {AuditStrings.FILTERS.DATE_RANGE}
          </Text>
          <Select.Root
            size="2"
            value={filters.dateRange || 'last-30-days'}
            onValueChange={(value) => onDateRangeChange(value as AuditFilters['dateRange'])}
            disabled={disabled}
          >
            <Select.Trigger className="audit-filter-sidebar__select-trigger" />
            <Select.Content>
              {DATE_RANGE_OPTIONS.map((opt) => (
                <Select.Item key={opt.value} value={opt.value}>
                  {opt.label}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </Box>

        {/* Categories */}
        <Box>
          <Text as="p" size="2" weight="bold" mb="2">
            {AuditStrings.FILTERS.CATEGORIES}
          </Text>
          <Flex direction="column">
            {CATEGORY_OPTIONS.map(({ value, label }) => (
              <Flex key={value} align="center" gap="2" mb="1">
                <Checkbox
                  id={`category-${value}`}
                  checked={filters.categories.includes(value)}
                  onCheckedChange={() => onCategoryToggle(value)}
                  disabled={disabled}
                  size="2"
                />
                <Text as="label" size="2" htmlFor={`category-${value}`}>
                  {label}
                </Text>
              </Flex>
            ))}
          </Flex>
        </Box>

        {/* Format */}
        <Box>
          <Text as="p" size="2" weight="bold" mb="2">
            {CommonStrings.LABELS.FORMAT}
          </Text>
          <Select.Root
            size="2"
            value={filters.repositoryType || '__all__'}
            onValueChange={(value) => onRepositoryTypeChange(value === '__all__' ? '' : value)}
            disabled={disabled}
          >
            <Select.Trigger className="audit-filter-sidebar__select-trigger" />
            <Select.Content>
              <Select.Item value="__all__">{AuditStrings.FILTERS.ALL_FORMATS}</Select.Item>
              {formatOptions.map((opt) => (
                <Select.Item key={opt.value} value={opt.value}>
                  {opt.label}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </Box>

        {/* Repository */}
        <Box>
          <Text as="p" size="2" weight="bold" mb="2">
            {AuditStrings.FILTERS.REPOSITORY}
          </Text>
          <Select.Root
            size="2"
            value={filters.repositoryName || '__all__'}
            onValueChange={(value) => onRepositoryNameChange(value === '__all__' ? '' : value)}
            disabled={disabled}
          >
            <Select.Trigger className="audit-filter-sidebar__select-trigger" />
            <Select.Content>
              <Select.Item value="__all__">{AuditStrings.FILTERS.ALL_REPOSITORIES}</Select.Item>
              {repoOptions.map((opt) => (
                <Select.Item key={opt.value} value={opt.value}>
                  {opt.label}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </Box>

        {/* Event Types */}
        <Box>
          <Text as="p" size="2" weight="bold" mb="2">
            {AuditStrings.FILTERS.EVENT_TYPES}
          </Text>
          <Flex direction="column">
            {EVENT_TYPE_OPTIONS.map(({ value, label }) => (
              <Flex key={value} align="center" gap="2" mb="1">
                <Checkbox
                  id={`eventtype-${value}`}
                  checked={filters.eventTypes.includes(value)}
                  onCheckedChange={() => onEventTypeToggle(value)}
                  disabled={disabled}
                  size="2"
                />
                <Text as="label" size="2" htmlFor={`eventtype-${value}`}>
                  {label}
                </Text>
              </Flex>
            ))}
          </Flex>
        </Box>

        {/* Initiator */}
        {/* Intentionally not disabled during loading to prevent focus loss while typing */}
        <Box>
          <Text as="p" size="2" weight="bold" mb="2">
            {AuditStrings.FILTERS.INITIATOR}
          </Text>
          <TextField.Root
            placeholder={AuditStrings.FILTERS.FILTER_INITIATOR_PLACEHOLDER}
            value={filters.initiator || ''}
            onChange={(e) => onInitiatorChange(e.target.value)}
            size="2"
          />
        </Box>
      </Flex>
    </Box>
  );
}
