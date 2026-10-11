import React from 'react';

import { SettingsFormSection, SettingsSelect, SettingsTextInput } from '../../../../../shared/form';

import { ConanProxyConfig, RepositoryFormData } from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

interface ConanFacetProps {
  formData: RepositoryFormData;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  isEdit: boolean;
}

const CONAN_VERSION_OPTIONS = [
  { value: 'V1', label: UIStrings.CONAN.PROTOCOL_VERSION.V1 },
  { value: 'V2', label: UIStrings.CONAN.PROTOCOL_VERSION.V2 },
];

/**
 * ConanFacet - Conan proxy configuration (`attributes.conan.conanVersion`, `conanProxy` in the REST API).
 *
 * Matches the Classic UI: the protocol version is chosen when the proxy is created (V2 by default)
 * and shown read-only afterwards, because a Conan 1 proxy cannot be migrated to Conan 2.
 */
export function ConanFacet({ formData, onNestedChange, isEdit }: ConanFacetProps) {
  const version = formData.conanProxy?.conanVersion ?? 'V2';
  const versionLabel = CONAN_VERSION_OPTIONS.find((option) => option.value === version)?.label ?? version;

  return (
    <SettingsFormSection
      title={UIStrings.CONAN.SECTION.title}
      description={UIStrings.CONAN.SECTION.description}
    >
      {isEdit ? (
        <SettingsTextInput
          name="conan-conanVersion"
          label={UIStrings.CONAN.PROTOCOL_VERSION.label}
          value={versionLabel}
          onChange={() => undefined}
          helpText={UIStrings.CONAN.PROTOCOL_VERSION.helpText}
          readOnly
        />
      ) : (
        <SettingsSelect
          name="conan-conanVersion"
          label={UIStrings.CONAN.PROTOCOL_VERSION.label}
          value={version}
          onChange={(value: string) =>
            onNestedChange('conanProxy', { conanVersion: value as ConanProxyConfig['conanVersion'] })}
          options={CONAN_VERSION_OPTIONS}
          helpText={UIStrings.CONAN.PROTOCOL_VERSION.helpText}
        />
      )}
    </SettingsFormSection>
  );
}

export default ConanFacet;
