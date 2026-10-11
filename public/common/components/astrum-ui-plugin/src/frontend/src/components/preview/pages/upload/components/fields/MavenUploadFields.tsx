

import React, { useMemo } from 'react';
import { Box, Callout } from '@radix-ui/themes';
import { Info, Package } from 'lucide-react';

import {
  SettingsTextInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsFormSection,
} from '../../../../shared/form';

import type { FormatFieldsProps, UploadComponentField } from '../../upload.types';
import {
  FORMAT_FIELD_STRINGS,
  MAVEN_FIELD_NAMES,
  MAVEN_PACKAGING_OPTIONS,
} from '../../upload.types';

import './MavenUploadFields.scss';
import UploadStrings from '../../../../constants/pages/upload/UploadStrings';

const MAVEN_COMPONENT_COORDS_GROUP = 'Component coordinates';

/**
 * Renders Maven-specific upload fields using shared form components.
 */
export function MavenUploadFields({
  fieldsByGroup,
  values,
  onChange,
  errors,
  disabledFields = new Set(),
  hasPomFile = false,
}: FormatFieldsProps): JSX.Element {
  const { maven } = FORMAT_FIELD_STRINGS;

  const coordFields = useMemo(() => {
    return fieldsByGroup[MAVEN_COMPONENT_COORDS_GROUP] || [];
  }, [fieldsByGroup]);

  const otherGroups = useMemo(() => {
    return Object.entries(fieldsByGroup).filter(
      ([groupName]) => groupName !== MAVEN_COMPONENT_COORDS_GROUP
    );
  }, [fieldsByGroup]);

  const coordsDisabled = hasPomFile;
  const generatePomValue = values[MAVEN_FIELD_NAMES.GENERATE_POM] as boolean;
  const isPackagingDisabled =
    disabledFields.has(MAVEN_FIELD_NAMES.PACKAGING) ||
    (!(generatePomValue || hasPomFile));

  const findField = (fieldName: string): UploadComponentField | undefined => {
    return coordFields.find((f) => f.name === fieldName);
  };

  const getHelp = (fieldName: string, defaultHelp: string) => {
    const field = findField(fieldName);
    return field?.helpText || defaultHelp;
  };

  const packagingField = findField(MAVEN_FIELD_NAMES.PACKAGING);
  const packagingOptions = packagingField?.selectOptions || [...MAVEN_PACKAGING_OPTIONS];

  return (
    <Box className="as-maven-upload-fields" data-testid="maven-upload-fields">
      <SettingsFormSection
        title={maven.groupName}
        icon={<Package size={18} />}
      >
        {hasPomFile && (
          <Callout.Root color="blue" className="as-maven-upload-fields__notice">
            <Callout.Icon>
              <Info size={16} />
            </Callout.Icon>
            <Callout.Text>{maven.pomDetectedInfo}</Callout.Text>
          </Callout.Root>
        )}

        <div className="as-maven-upload-fields__grid">
          <SettingsTextInput
            name="groupId"
            label={maven.groupIdLabel}
            value={(values[MAVEN_FIELD_NAMES.GROUP_ID] as string) || ''}
            onChange={(val) => onChange(MAVEN_FIELD_NAMES.GROUP_ID, val)}
            disabled={coordsDisabled || disabledFields.has(MAVEN_FIELD_NAMES.GROUP_ID)}
            placeholder="com.example"
            helpText={getHelp(MAVEN_FIELD_NAMES.GROUP_ID, maven.groupIdHelp)}
            error={errors[MAVEN_FIELD_NAMES.GROUP_ID] || ''}
            required
          />

          <SettingsTextInput
            name="artifactId"
            label={maven.artifactIdLabel}
            value={(values[MAVEN_FIELD_NAMES.ARTIFACT_ID] as string) || ''}
            onChange={(val) => onChange(MAVEN_FIELD_NAMES.ARTIFACT_ID, val)}
            disabled={coordsDisabled || disabledFields.has(MAVEN_FIELD_NAMES.ARTIFACT_ID)}
            placeholder="my-library"
            helpText={getHelp(MAVEN_FIELD_NAMES.ARTIFACT_ID, maven.artifactIdHelp)}
            error={errors[MAVEN_FIELD_NAMES.ARTIFACT_ID] || ''}
            required
          />

          <SettingsTextInput
            name="version"
            label={maven.versionLabel}
            value={(values[MAVEN_FIELD_NAMES.VERSION] as string) || ''}
            onChange={(val) => onChange(MAVEN_FIELD_NAMES.VERSION, val)}
            disabled={coordsDisabled || disabledFields.has(MAVEN_FIELD_NAMES.VERSION)}
            placeholder="1.0.0"
            helpText={getHelp(MAVEN_FIELD_NAMES.VERSION, maven.versionHelp)}
            error={errors[MAVEN_FIELD_NAMES.VERSION] || ''}
            required
          />

          <SettingsSelect
            name="packaging"
            label={maven.packagingLabel}
            value={(values[MAVEN_FIELD_NAMES.PACKAGING] as string) || 'jar'}
            onChange={(val) => onChange(MAVEN_FIELD_NAMES.PACKAGING, val)}
            options={packagingOptions.map((o) => ({ value: o, label: o }))}
            helpText={getHelp(MAVEN_FIELD_NAMES.PACKAGING, maven.packagingHelp)}
            disabled={isPackagingDisabled}
          />

          <SettingsTextInput
            name="extension"
            label={maven.extensionLabel}
            value={(values[MAVEN_FIELD_NAMES.EXTENSION] as string) || ''}
            onChange={(val) => onChange(MAVEN_FIELD_NAMES.EXTENSION, val)}
            disabled={disabledFields.has(MAVEN_FIELD_NAMES.EXTENSION)}
            placeholder="jar"
            helpText={getHelp(MAVEN_FIELD_NAMES.EXTENSION, maven.extensionHelp)}
          />

          <SettingsTextInput
            name="classifier"
            label={maven.classifierLabel}
            value={(values[MAVEN_FIELD_NAMES.CLASSIFIER] as string) || ''}
            onChange={(val) => onChange(MAVEN_FIELD_NAMES.CLASSIFIER, val)}
            disabled={disabledFields.has(MAVEN_FIELD_NAMES.CLASSIFIER)}
            placeholder="sources"
            helpText={getHelp(MAVEN_FIELD_NAMES.CLASSIFIER, maven.classifierHelp)}
          />
        </div>

        <SettingsCheckbox
          name="generate-pom"
          label={maven.generatePomLabel}
          checked={generatePomValue}
          onChange={(checked) =>
            onChange(MAVEN_FIELD_NAMES.GENERATE_POM, checked)
          }
          disabled={hasPomFile || disabledFields.has(MAVEN_FIELD_NAMES.GENERATE_POM)}
          description={getHelp(MAVEN_FIELD_NAMES.GENERATE_POM, maven.generatePomHelp)}
        />
      </SettingsFormSection>

      {otherGroups.map(([groupName, fields]) => (
        <SettingsFormSection key={groupName} title={groupName === 'Other' ? UploadStrings.GENERIC_FIELDS.OTHER_GROUP : groupName}>
          {fields.map((field: UploadComponentField) => (
            <SettingsTextInput
              key={field.name}
              name={field.name}
              label={field.displayName || field.name}
              value={(values[field.name] as string) || ''}
              onChange={(val) => onChange(field.name, val)}
              disabled={disabledFields.has(field.name)}
              helpText={field.helpText || ''}
              error={errors[field.name] || ''}
              required={!field.optional}
            />
          ))}
        </SettingsFormSection>
      ))}
    </Box>
  );
}

export default MavenUploadFields;
