

import React, { useState, useCallback } from 'react';
import { Box, Flex, TextField, Button, Text, Select } from '@radix-ui/themes';
import { X, ChevronDown, ChevronUp } from 'lucide-react';
import type { YumSearchFilters as FilterValues } from './yum.types';

const ANY_VALUE = '__any__';

const ARCHITECTURES = [
  { value: ANY_VALUE, label: 'Any Architecture' },
  { value: 'x86_64', label: 'x86_64 (64-bit)' },
  { value: 'i686', label: 'i686 (32-bit)' },
  { value: 'noarch', label: 'noarch (Any)' },
  { value: 'aarch64', label: 'aarch64 (ARM 64-bit)' },
];

export interface YumSearchFiltersProps {
  values: FilterValues;
  onChange: (values: FilterValues) => void;
  onSearch: () => void;
  onClear: () => void;
  loading?: boolean;
}

export function YumSearchFilters({
  values,
  onChange,
  onSearch,
  onClear,
  loading = false,
}: YumSearchFiltersProps): JSX.Element {
  const [showAdvanced, setShowAdvanced] = useState(false);

  const handleInputChange = useCallback((field: keyof FilterValues) => (
    event: React.ChangeEvent<HTMLInputElement>
  ): void => {
    onChange({
      ...values,
      [field]: event.target.value || undefined,
    });
  }, [values, onChange]);

  const handleSelectChange = useCallback((field: keyof FilterValues) => (value: string): void => {
    onChange({
      ...values,
      [field]: value || undefined,
    });
  }, [values, onChange]);

  const handleKeyDown = useCallback((event: React.KeyboardEvent): void => {
    if (event.key === 'Enter') {
      onSearch();
    }
  }, [onSearch]);

  const hasAdvancedFilters = Boolean(values.version || values.architecture);

  return (
    <Box p="4" style={{ backgroundColor: 'var(--gray-2)', borderRadius: 'var(--radius-3)' }}>
      <Flex direction="column" gap="3">
        <Flex justify="between" align="center">
          <Text size="2" weight="medium">Additional Filters</Text>
          <Button variant="ghost" size="1" onClick={() => setShowAdvanced(!showAdvanced)}>
            {showAdvanced ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
            {showAdvanced ? 'Hide' : 'Show'}
          </Button>
        </Flex>

        {showAdvanced && (
          <Flex direction="column" gap="3">
            <Flex gap="3" wrap="wrap">
              <Box style={{ flex: '1 1 150px' }}>
                <Text as="label" size="1" color="gray" mb="1">Version</Text>
                <TextField.Root
                  placeholder="e.g., 1.2.3-1.el8"
                  value={values.version ?? ''}
                  onChange={handleInputChange('version')}
                  onKeyDown={handleKeyDown}
                  disabled={loading}
                />
              </Box>

              <Box style={{ flex: '1 1 180px' }}>
                <Text as="label" size="1" color="gray" mb="1">Architecture</Text>
                <Select.Root
                  value={values.architecture || ANY_VALUE}
                  onValueChange={(v) => handleSelectChange('architecture')(v === ANY_VALUE ? '' : v)}
                  disabled={loading}
                >
                  <Select.Trigger placeholder="Select architecture" />
                  <Select.Content>
                    {ARCHITECTURES.map((arch) => (
                      <Select.Item key={arch.value} value={arch.value}>
                        {arch.label}
                      </Select.Item>
                    ))}
                  </Select.Content>
                </Select.Root>
              </Box>
            </Flex>

            <Flex justify="end" gap="2">
              {hasAdvancedFilters && (
                <Button variant="ghost" onClick={onClear} disabled={loading}>
                  <X size={14} />
                  Clear Filters
                </Button>
              )}
            </Flex>
          </Flex>
        )}
      </Flex>
    </Box>
  );
}

export default YumSearchFilters;
