

import React from 'react';
import { Box, Button, Checkbox, Flex, Text } from '@radix-ui/themes';
import { RefreshCw } from 'lucide-react';

import type { TagsFilters } from './hooks/useFilteredTags';

const COMPONENT_COUNT_OPTIONS = [
  { value: '0', label: 'Empty (0)' },
  { value: '1-10', label: '1-10' },
  { value: '11-100', label: '11-100' },
  { value: '101-1000', label: '101-1000' },
  { value: '1000+', label: '1000+' },
];

const ACTIVITY_OPTIONS = [
  { value: 30, label: 'Active (< 30 days)' },
  { value: 90, label: 'Stale (30-90 days)' },
  { value: 9000, label: 'Abandoned (90+ days)' },
];

export interface TagsFilterSidebarProps {
  filters: TagsFilters;
  onToggleComponentCountRange: (range: string) => void;
  onToggleActivityDays: (days: number) => void;
  onClearAllFilters: () => void;
  hasActiveFilters: boolean;
  disabled?: boolean;
}

/**
 * Filter sidebar for Tags page. Matches Browse filter structure with Component Count and Activity filters.
 */
export function TagsFilterSidebar({
  filters,
  onToggleComponentCountRange,
  onToggleActivityDays,
  onClearAllFilters,
  hasActiveFilters,
  disabled = false,
}: TagsFilterSidebarProps): JSX.Element {
  return (
    <Box p="1" pt="4">
      <Flex align="center" justify="start" mb="4">
        <Button
          variant="outline"
          color="gray"
          size="2"
          onClick={onClearAllFilters}
          disabled={disabled || !hasActiveFilters}
        >
          <RefreshCw size={12} />
          Reset filters
        </Button>
      </Flex>

      <Flex direction="column" gap="4">
        {/* Component Count */}
        <Box mt="1">
          <Text size="2" weight="bold" as="div" mb="3">
            Component Count
          </Text>
          <Flex direction="column" gap="2">
            {COMPONENT_COUNT_OPTIONS.map((opt) => (
              <Flex key={opt.value} align="center" gap="2">
                <Checkbox
                  checked={filters.componentCountRanges.includes(opt.value)}
                  onCheckedChange={() => onToggleComponentCountRange(opt.value)}
                  disabled={disabled}
                />
                <Text size="2">{opt.label}</Text>
              </Flex>
            ))}
          </Flex>
        </Box>

        {/* Activity */}
        <Box mt="1">
          <Text size="2" weight="bold" as="div" mb="3">
            Activity
          </Text>
          <Flex direction="column" gap="2">
            {ACTIVITY_OPTIONS.map((opt) => (
              <Flex key={opt.value} align="center" gap="2">
                <Checkbox
                  checked={filters.activityDays.includes(opt.value)}
                  onCheckedChange={() => onToggleActivityDays(opt.value)}
                  disabled={disabled}
                />
                <Text size="2">{opt.label}</Text>
              </Flex>
            ))}
          </Flex>
        </Box>
      </Flex>
    </Box>
  );
}
