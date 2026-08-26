

import React from 'react';
import { Box } from '@radix-ui/themes';
import { File } from 'lucide-react';

import { SettingsTextInput, SettingsFormSection } from '../../../../shared/form';
import { UploadField } from './GenericUploadFields';

import type { FormatFieldsProps, UploadComponentField } from '../../upload.types';
import { FORMAT_FIELD_STRINGS } from '../../upload.types';

import './RawUploadFields.scss';

const RAW_FIELD_NAMES = {
  DIRECTORY: 'directory',
  FILENAME: 'filename',
} as const;

/**
 * Renders raw format upload fields using shared form components.
 */
export function RawUploadFields({
  fieldsByGroup,
  values,
  onChange,
  errors,
  disabledFields = new Set(),
}: FormatFieldsProps): JSX.Element {
  const { raw } = FORMAT_FIELD_STRINGS;

  const allFields = Object.values(fieldsByGroup).flat();
  const directoryField = allFields.find((f) => f.name === RAW_FIELD_NAMES.DIRECTORY);
  const filenameField = allFields.find((f) => f.name === RAW_FIELD_NAMES.FILENAME);
  const otherFields = allFields.filter(
    (f) => f.name !== RAW_FIELD_NAMES.DIRECTORY && f.name !== RAW_FIELD_NAMES.FILENAME
  );

  return (
    <Box className="as-raw-upload-fields" data-testid="raw-upload-fields">
      <SettingsFormSection title="Raw File Details" icon={<File size={18} />}>
        <Box className="as-raw-upload-fields__grid">
          <SettingsTextInput
            name="raw-directory"
            label={raw.directoryLabel}
            value={(values[RAW_FIELD_NAMES.DIRECTORY] as string) || ''}
            onChange={(val) => onChange(RAW_FIELD_NAMES.DIRECTORY, val)}
            disabled={disabledFields.has(RAW_FIELD_NAMES.DIRECTORY)}
            placeholder="/path/to/directory"
            helpText={raw.directoryHelp}
            error={errors[RAW_FIELD_NAMES.DIRECTORY] || ''}
            required={directoryField ? !directoryField.optional : false}
          />

          <SettingsTextInput
            name="raw-filename"
            label={raw.filenameLabel}
            value={(values[RAW_FIELD_NAMES.FILENAME] as string) || ''}
            onChange={(val) => onChange(RAW_FIELD_NAMES.FILENAME, val)}
            disabled={disabledFields.has(RAW_FIELD_NAMES.FILENAME)}
            placeholder="filename.ext"
            helpText={raw.filenameHelp}
            error={errors[RAW_FIELD_NAMES.FILENAME] || ''}
            required={filenameField ? !filenameField.optional : false}
          />
        </Box>

        {otherFields.length > 0 && (
          <Box className="as-raw-upload-fields__other-fields">
            {otherFields.map((field: UploadComponentField) => (
              <UploadField
                key={field.name}
                field={field}
                value={values[field.name] ?? (field.type === 'BOOLEAN' ? false : '')}
                onChange={(value) => onChange(field.name, value)}
                error={errors[field.name]}
                disabled={disabledFields.has(field.name)}
              />
            ))}
          </Box>
        )}
      </SettingsFormSection>
    </Box>
  );
}

export default RawUploadFields;
