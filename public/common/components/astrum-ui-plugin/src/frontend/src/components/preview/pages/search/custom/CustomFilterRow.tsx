

import React from 'react';
import { Flex, TextField, Select, IconButton, Tooltip } from '@radix-ui/themes';
import { X } from 'lucide-react';
import type { CustomFilter, FilterField, FilterOperator } from './custom.types';
import {
  FILTER_FIELD_OPTIONS,
  FILTER_OPERATOR_OPTIONS,
  filterFieldOptions,
} from './custom.types';
import S from '../../../constants/pages/search/CustomSearchStrings';

export interface CustomFilterRowProps {
  filter: CustomFilter;
  onUpdate: (id: string, updates: Partial<CustomFilter>) => void;
  onRemove: (id: string) => void;
  isOnlyFilter?: boolean;
  disabled?: boolean;
  /** Whether the edition offers tags as a field */
  proEdition?: boolean;
}

export function CustomFilterRow({
  filter,
  onUpdate,
  onRemove,
  isOnlyFilter = false,
  disabled = false,
  proEdition = false,
}: CustomFilterRowProps): JSX.Element {
  const handleFieldChange = (value: string): void => {
    onUpdate(filter.id, { field: value as FilterField });
  };

  const handleOperatorChange = (value: string): void => {
    onUpdate(filter.id, { operator: value as FilterOperator });
  };

  const handleValueChange = (e: React.ChangeEvent<HTMLInputElement>): void => {
    onUpdate(filter.id, { value: e.target.value });
  };

  const handleRemove = (): void => {
    onRemove(filter.id);
  };

  const currentFieldOption = FILTER_FIELD_OPTIONS.find(
    (opt) => opt.value === filter.field
  );
  const placeholder = currentFieldOption?.placeholder || S.VALUE_PLACEHOLDER;
  const fieldOptions = filterFieldOptions(proEdition);

  return (
    <Flex gap="2" align="center" wrap="wrap">
      <Select.Root value={filter.field} onValueChange={handleFieldChange} disabled={disabled}>
        <Select.Trigger style={{ minWidth: '140px' }} aria-label={S.FIELD_LABEL} />
        <Select.Content>
          {fieldOptions.map((opt) => (
            <Select.Item key={opt.value} value={opt.value}>
              {opt.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>

      <Select.Root value={filter.operator} onValueChange={handleOperatorChange} disabled={disabled}>
        <Select.Trigger style={{ minWidth: '120px' }} aria-label={S.OPERATOR_LABEL} />
        <Select.Content>
          {FILTER_OPERATOR_OPTIONS.map((opt) => (
            <Select.Item key={opt.value} value={opt.value}>
              {opt.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>

      <TextField.Root
        style={{ flex: '1 1 200px' }}
        value={filter.value}
        onChange={handleValueChange}
        placeholder={placeholder}
        aria-label={S.VALUE_LABEL}
        disabled={disabled}
      />

      <Tooltip content={isOnlyFilter ? S.ONLY_FILTER : S.REMOVE_FILTER}>
        <IconButton
          variant="ghost"
          color="gray"
          onClick={handleRemove}
          disabled={disabled || isOnlyFilter}
          aria-label={S.REMOVE_FILTER}
        >
          <X size={16} />
        </IconButton>
      </Tooltip>
    </Flex>
  );
}

export default CustomFilterRow;
