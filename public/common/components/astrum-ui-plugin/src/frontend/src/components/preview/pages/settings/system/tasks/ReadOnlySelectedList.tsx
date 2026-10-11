
import React from 'react';
import { Box, Text } from '@radix-ui/themes';
import TasksStrings from '../../../../constants/pages/settings/system/TasksStrings';

interface ReadOnlySelectedListProps {
  label?: string;
  helpText?: string;
  values: string[];
  emptyText?: string;
  testId?: string;
}

/**
 * Read-only "Selected"-only display for a task field that mirrors Classic renderExecutePlanFields:
 * the blob-store / repository values are shown without an Available column or transfer controls.
 * Presentational only — no data fetching, no editing.
 */
export function ReadOnlySelectedList({ label, helpText, values, emptyText, testId }: ReadOnlySelectedListProps): React.ReactElement {
  return (
    <Box className="dynamic-field read-only-selected-list">
      {label && (
        <Text as="div" size="2" weight="medium" className="dynamic-field__label">{label}</Text>
      )}
      {helpText && (
        <Text as="p" size="1" color="gray" className="dynamic-field__help">{helpText}</Text>
      )}
      <Text as="div" size="2" weight="medium" className="read-only-selected-list__heading" mt="1">{TasksStrings.SELECTED_LIST.SELECTED}</Text>
      <Box
        role="list"
        className="read-only-selected-list__box"
        data-testid={testId}
        style={{
          background: 'var(--gray-3)',
          border: '1px solid var(--gray-5)',
          borderRadius: 'var(--radius-2)',
          padding: 'var(--space-2)',
          minHeight: '120px',
          color: 'var(--gray-11)',
        }}
      >
        {values.length === 0 && emptyText && (
          <Text as="div" size="2" color="gray" className="read-only-selected-list__empty">{emptyText}</Text>
        )}
        {values.map((v) => (
          <div key={v} role="listitem" className="read-only-selected-list__item">{v}</div>
        ))}
      </Box>
    </Box>
  );
}

export default ReadOnlySelectedList;
