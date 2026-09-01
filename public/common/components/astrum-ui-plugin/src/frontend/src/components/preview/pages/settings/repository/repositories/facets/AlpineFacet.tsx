

import React from 'react';

import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsPasswordInput,
} from '../../../../../shared/form';

import { RepositoryFormData, RepositoryFormErrors } from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

interface AlpineFacetProps {
  formData: RepositoryFormData;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  errors?: RepositoryFormErrors;
  repoType: 'hosted' | 'proxy' | 'group';
}

/**
 * AlpineFacet - Alpine Linux package repository configuration
 *
 * Fields:
 * - RSA Signing keypair and passphrase (hosted and group)
 */
export function AlpineFacet({
  formData,
  onNestedChange,
  errors,
  repoType,
}: AlpineFacetProps) {
  // Show signing for hosted and group repositories
  const showSigning = repoType === 'hosted' || repoType === 'group';

  return (
    <>
      {showSigning && (
        <SettingsFormSection
          title={UIStrings.ALPINE.SIGNING.title}
          description={UIStrings.ALPINE.SIGNING.description}
        >
          <SettingsTextInput
            name="alpine-keypair"
            label={UIStrings.ALPINE.SIGNING.KEYPAIR.label}
            value={formData.alpineSigning?.keypair || ''}
            onChange={(value) => onNestedChange('alpineSigning', { keypair: value })}
            helpText={UIStrings.ALPINE.SIGNING.KEYPAIR.helpText}
            required
          />

          <SettingsPasswordInput
            name="alpine-passphrase"
            label={UIStrings.ALPINE.SIGNING.PASSPHRASE.label}
            value={formData.alpineSigning?.passphrase || ''}
            onChange={(value) => onNestedChange('alpineSigning', { passphrase: value })}
            helpText={UIStrings.ALPINE.SIGNING.PASSPHRASE.helpText}
          />
        </SettingsFormSection>
      )}
    </>
  );
}

export default AlpineFacet;
