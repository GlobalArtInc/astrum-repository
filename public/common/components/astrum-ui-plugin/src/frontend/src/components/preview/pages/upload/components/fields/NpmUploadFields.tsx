

import React from 'react';
import { Box, Text } from '@radix-ui/themes';
import { Package } from 'lucide-react';

import { SettingsTextInput, SettingsFormSection } from '../../../../shared/form';
import { UploadField } from './GenericUploadFields';

import type { FormatFieldsProps, UploadComponentField } from '../../upload.types';
import { FORMAT_FIELD_STRINGS } from '../../upload.types';

import './NpmUploadFields.scss';
import UploadStrings from '../../../../constants/pages/upload/UploadStrings';

const NPM_FIELD_NAMES = {
  PACKAGE_NAME: 'name',
  VERSION: 'version',
} as const;

/**
 * Renders npm-specific upload fields using shared form components.
 */
export function NpmUploadFields({
  fieldsByGroup,
  values,
  onChange,
  errors,
  disabledFields = new Set(),
}: FormatFieldsProps): JSX.Element {
  const { npm } = FORMAT_FIELD_STRINGS;

  const allFields = Object.values(fieldsByGroup).flat();
  const packageNameField = allFields.find((f) => f.name === NPM_FIELD_NAMES.PACKAGE_NAME);
  const versionField = allFields.find((f) => f.name === NPM_FIELD_NAMES.VERSION);
  const otherFields = allFields.filter(
    (f) => f.name !== NPM_FIELD_NAMES.PACKAGE_NAME && f.name !== NPM_FIELD_NAMES.VERSION
  );

  if (allFields.length === 0) {
    return (
      <Box className="as-npm-upload-fields" data-testid="npm-upload-fields">
        <SettingsFormSection title={UploadStrings.NPM_FIELDS.NPM_PACKAGE} icon={<Package size={18} />}>
          <Text size="2" color="gray">
            {UploadStrings.NPM_FIELDS.UPLOAD_NPM_PACKAGE_TARBALL_TGZ}
          </Text>
        </SettingsFormSection>
      </Box>
    );
  }

  return (
    <Box className="as-npm-upload-fields" data-testid="npm-upload-fields">
      <SettingsFormSection title={UploadStrings.NPM_FIELDS.NPM_PACKAGE_DETAILS} icon={<Package size={18} />}>
        <Box className="as-npm-upload-fields__grid">
          {packageNameField && (
            <SettingsTextInput
              name="npm-package-name"
              label={npm.packageNameLabel}
              value={(values[NPM_FIELD_NAMES.PACKAGE_NAME] as string) || ''}
              onChange={(val) => onChange(NPM_FIELD_NAMES.PACKAGE_NAME, val)}
              disabled={disabledFields.has(NPM_FIELD_NAMES.PACKAGE_NAME)}
              placeholder="@scope/package-name"
              helpText={npm.packageNameHelp}
              error={errors[NPM_FIELD_NAMES.PACKAGE_NAME] || ''}
              required={!packageNameField.optional}
            />
          )}

          {versionField && (
            <SettingsTextInput
              name="npm-version"
              label={npm.versionLabel}
              value={(values[NPM_FIELD_NAMES.VERSION] as string) || ''}
              onChange={(val) => onChange(NPM_FIELD_NAMES.VERSION, val)}
              disabled={disabledFields.has(NPM_FIELD_NAMES.VERSION)}
              placeholder="1.0.0"
              helpText={npm.versionHelp}
              error={errors[NPM_FIELD_NAMES.VERSION] || ''}
              required={!versionField.optional}
            />
          )}
        </Box>

        {otherFields.length > 0 && (
          <Box className="as-npm-upload-fields__other-fields">
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

export default NpmUploadFields;
