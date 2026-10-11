

import React from 'react';
import { SettingsTextInput, SettingsSelect, SettingsCheckbox } from '../../shared/form';
import type { UploadFieldDefinition } from './upload.types';
import UploadStrings from '../../constants/pages/upload/UploadStrings';

/**
 * Labels and hints of the fields the server describes by name only.
 */
const KNOWN_FIELDS: Record<string, { label: string; help: string }> = {
  classifier: {
    label: UploadStrings.TYPES.CLASSIFIER,
    help: UploadStrings.TYPES.OPTIONAL_CLASSIFIER_E_G_SOURCES,
  },
  extension: {
    label: UploadStrings.TYPES.EXTENSION,
    help: UploadStrings.GENERIC_FIELDS.FILE_EXTENSION_ASSET_E_G,
  },
  filename: {
    label: UploadStrings.TYPES.FILENAME,
    help: UploadStrings.TYPES.FILENAME_OPTIONAL_USES_UPLOADED_FILENAME,
  },
};

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
  const known = KNOWN_FIELDS[field.name];
  const commonProps = {
    name: field.name,
    label: field.displayName || known?.label || field.name,
    helpText: field.helpText || known?.help,
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
