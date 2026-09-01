

import React from 'react';

import {
  SettingsFormSection,
  SettingsSelect,
  SettingsCheckbox,
} from '../../../../../shared/form';

import {
  RepositoryFormData,
  RepositoryFormErrors,
  WRITE_POLICY_OPTIONS,
  WritePolicy,
} from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

// Formats that do not support namespace confusion protection via Firewall.
// Matches classic UI: GenericHostedConfiguration.jsx UNSUPPORTED_NAMESPACE_CONFUSION_FORMATS
const UNSUPPORTED_NAMESPACE_CONFUSION_FORMATS = ['docker'];

interface HostedFacetProps {
  formData: RepositoryFormData;
  onChange: (updates: Partial<RepositoryFormData>) => void;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  errors?: RepositoryFormErrors;
}

/**
 * HostedFacet - Hosted repository specific settings
 */
export function HostedFacet({
  formData,
  onChange,
  onNestedChange,
  errors,
}: HostedFacetProps) {
  const handleWritePolicyChange = (value: string) => {
    onNestedChange('storage', { writePolicy: value as WritePolicy });
  };

  const handleProprietaryChange = (checked: boolean) => {
    onNestedChange('component', { proprietaryComponents: checked });
  };

  const supportsNamespaceConfusion = !UNSUPPORTED_NAMESPACE_CONFUSION_FORMATS.includes(formData.format);

  return (
    <SettingsFormSection title={UIStrings.HOSTED.SECTION.title}>
      <SettingsSelect
        name="storage-writePolicy"
        label={UIStrings.HOSTED.DEPLOYMENT_POLICY.label}
        value={formData.storage?.writePolicy || 'ALLOW_ONCE'}
        onChange={handleWritePolicyChange}
        options={WRITE_POLICY_OPTIONS}
        helpText={UIStrings.HOSTED.DEPLOYMENT_POLICY.helpText}
      />

      {supportsNamespaceConfusion && (
        <SettingsCheckbox
          name="component-proprietaryComponents"
          label={UIStrings.HOSTED.PROPRIETARY_COMPONENTS.label}
          checked={formData.component?.proprietaryComponents ?? false}
          onChange={handleProprietaryChange}
          description={UIStrings.HOSTED.PROPRIETARY_COMPONENTS.description}
        />
      )}
    </SettingsFormSection>
  );
}

export default HostedFacet;

