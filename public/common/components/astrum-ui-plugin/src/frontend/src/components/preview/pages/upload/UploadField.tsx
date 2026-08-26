

import React from 'react';
import { SettingsTextInput, SettingsSelect, SettingsCheckbox } from '../../shared/form';
import type { UploadFieldDefinition } from './upload.types';

/**
 * Renders an individual upload field based on its definition.
 */
export function UploadField({
  field,
  value,
  onChange,
  error,
  onBlur,
}: {
  field: UploadFieldDefinition;
  value: string | boolean;
  onChange: (value: string | boolean) => void;
  error?: string | null;
  onBlur?: () => void;
}): JSX.Element {
  const commonProps = {
    name: field.name,
    label: field.displayName || field.name,
    helpText: field.helpText,
    required: !field.optional,
  };

  if (field.type === 'BOOLEAN') {
    return (
      <SettingsCheckbox
        {...commonProps}
        checked={value as boolean}
        onChange={onChange}
        description={field.helpText}
      />
    );
  }

  if (field.type === 'SELECT' && field.selectOptions) {
    return (
      <SettingsSelect
        {...commonProps}
        value={value as string}
        onChange={onChange}
        options={field.selectOptions.map((o) => ({ value: o, label: o }))}
        error={error || ''}
      />
    );
  }

  return (
    <SettingsTextInput
      {...commonProps}
      value={value as string}
      onChange={onChange}
      onBlur={onBlur}
      error={error || ''}
    />
  );
}

export default UploadField;
