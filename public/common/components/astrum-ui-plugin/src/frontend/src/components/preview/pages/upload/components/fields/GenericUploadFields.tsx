

import React from 'react';
import { Box } from '@radix-ui/themes';

import {
  SettingsTextInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsFormSection,
} from '../../../../shared/form';

import type {
  UploadFieldProps,
  FormatFieldsProps,
  UploadComponentField,
} from '../../upload.types';

import './GenericUploadFields.scss';
import UploadStrings from '../../../../constants/pages/upload/UploadStrings';

const FIELD_HELP_TEXT: Record<string, string> = {
  groupId: UploadStrings.GENERIC_FIELDS.MAVEN_GROUP_IDENTIFIER_E_G,
  artifactId: UploadStrings.GENERIC_FIELDS.MAVEN_ARTIFACT_NAME_E_G,
  version: UploadStrings.GENERIC_FIELDS.ARTIFACT_VERSION_E_G_1,
  packaging: UploadStrings.GENERIC_FIELDS.PACKAGING_TYPE_E_G_JAR,
  classifier: UploadStrings.GENERIC_FIELDS.OPTIONAL_CLASSIFIER_VARIANTS_E_G,
  extension: 'File extension for this asset (e.g., jar, pom)',
  'generate-pom': UploadStrings.GENERIC_FIELDS.AUTOMATICALLY_CREATES_POM_FILE_SPECIFIED,
  tag: 'Optional tag to categorize components',
  directory: UploadStrings.GENERIC_FIELDS.TARGET_DIRECTORY_PATH_E_G,
  id: 'Unique package identifier (e.g., MyCompany.MyPackage)',
  name: 'Package name using lowercase with hyphens (e.g., my-package)',
  filename: 'Name of the file being uploaded',
  path: 'Storage path within the repository',
  'docker.imageName': UploadStrings.GENERIC_FIELDS.DOCKER_IMAGE_NAME_E_G,
  'docker.imageTag': UploadStrings.GENERIC_FIELDS.DOCKER_IMAGE_TAG_E_G,
};

const FIELD_PLACEHOLDERS: Record<string, string> = {
  groupId: 'com.example.myapp',
  artifactId: 'my-library',
  version: '1.0.0',
  packaging: 'jar',
  classifier: 'sources',
  extension: 'jar',
  tag: 'release-candidate',
  directory: '/path/to/files',
  filename: 'my-file.txt',
  id: 'MyCompany.MyPackage',
  name: 'my-package',
  path: '/path/to/asset',
  'docker.imageName': 'mycompany/myimage',
  'docker.imageTag': 'latest',
};

function getHelpText(fieldName: string, backendHelp?: string): string {
  return backendHelp || FIELD_HELP_TEXT[fieldName] || '';
}

/**
 * Renders a single form field using the shared Settings* components.
 */
export function UploadField({
  field,
  value,
  onChange,
  error,
  disabled = false,
}: UploadFieldProps): JSX.Element {
  const label = field.displayName || field.name;
  const helpText = getHelpText(field.name, field.helpText);
  const placeholder = FIELD_PLACEHOLDERS[field.name] || '';

  if (field.type === 'BOOLEAN') {
    return (
      <SettingsCheckbox
        name={field.name}
        label={label}
        checked={value as boolean}
        onChange={(checked: boolean) => onChange(checked)}
        description={helpText}
        disabled={disabled}
      />
    );
  }

  if (field.type === 'SELECT' && field.selectOptions) {
    return (
      <SettingsSelect
        name={field.name}
        label={label}
        value={value as string}
        onChange={(val: string) => onChange(val)}
        options={field.selectOptions.map((o) => ({ value: o, label: o }))}
        helpText={helpText}
        error={error || ''}
        required={!field.optional}
        disabled={disabled}
      />
    );
  }

  return (
    <SettingsTextInput
      name={field.name}
      label={label}
      value={value as string}
      onChange={(val: string) => onChange(val)}
      helpText={helpText}
      error={error || ''}
      required={!field.optional}
      disabled={disabled}
      placeholder={placeholder}
    />
  );
}

/**
 * Renders all fields for a generic/unknown format using shared form components.
 */
export function GenericUploadFields({
  fieldsByGroup,
  values,
  onChange,
  errors,
  disabledFields = new Set(),
}: FormatFieldsProps): JSX.Element {
  const groupNames = Object.keys(fieldsByGroup);

  if (groupNames.length === 0) {
    return <></>;
  }

  return (
    <Box className="as-generic-upload-fields" data-testid="generic-upload-fields">
      {groupNames.map((groupName) => {
        const fields = fieldsByGroup[groupName];

        return (
          <SettingsFormSection
            key={groupName}
            title={groupName === 'Other' ? UploadStrings.GENERIC_FIELDS.OTHER_GROUP : groupName}
          >
            {fields.map((field: UploadComponentField) => (
              <UploadField
                key={field.name}
                field={field}
                value={values[field.name] ?? (field.type === 'BOOLEAN' ? false : '')}
                onChange={(value) => onChange(field.name, value)}
                error={errors[field.name]}
                disabled={disabledFields.has(field.name)}
              />
            ))}
          </SettingsFormSection>
        );
      })}
    </Box>
  );
}

export default GenericUploadFields;
