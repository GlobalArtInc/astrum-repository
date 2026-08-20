

import React, { useState, useCallback } from 'react';
import { Box, Flex, TextField, Button, Text } from '@radix-ui/themes';
import { X, ChevronDown, ChevronUp } from 'lucide-react';

export interface FilterValues {
  groupId: string;
  artifactId: string;
  repository: string;
}

export interface GASearchFiltersProps {
  values: FilterValues;
  onChange: (values: FilterValues) => void;
  onSearch: () => void;
  onClear: () => void;
  loading?: boolean;
}

export function GASearchFilters({
  values,
  onChange,
  onSearch,
  onClear,
  loading = false,
}: GASearchFiltersProps): JSX.Element {
  const [showAdvanced, setShowAdvanced] = useState(false);

  const handleInputChange = useCallback((field: keyof FilterValues) => (
    event: React.ChangeEvent<HTMLInputElement>
  ): void => {
    onChange({
      ...values,
      [field]: event.target.value,
    });
  }, [values, onChange]);

  const handleKeyDown = useCallback((event: React.KeyboardEvent): void => {
    if (event.key === 'Enter') {
      onSearch();
    }
  }, [onSearch]);

  const hasFilters = Boolean(values.groupId || values.artifactId || values.repository);

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
              <Box style={{ flex: '1 1 200px' }}>
                <Text as="label" size="1" color="gray" mb="1">Group ID</Text>
                <TextField.Root
                  placeholder="e.g., org.apache.commons"
                  value={values.groupId}
                  onChange={handleInputChange('groupId')}
                  onKeyDown={handleKeyDown}
                  disabled={loading}
                />
              </Box>

              <Box style={{ flex: '1 1 200px' }}>
                <Text as="label" size="1" color="gray" mb="1">Artifact ID</Text>
                <TextField.Root
                  placeholder="e.g., commons-lang3"
                  value={values.artifactId}
                  onChange={handleInputChange('artifactId')}
                  onKeyDown={handleKeyDown}
                  disabled={loading}
                />
              </Box>

              <Box style={{ flex: '1 1 200px' }}>
                <Text as="label" size="1" color="gray" mb="1">Repository</Text>
                <TextField.Root
                  placeholder="e.g., maven-central"
                  value={values.repository}
                  onChange={handleInputChange('repository')}
                  onKeyDown={handleKeyDown}
                  disabled={loading}
                />
              </Box>
            </Flex>

            <Flex justify="end" gap="2">
              {hasFilters && (
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

export default GASearchFilters;


