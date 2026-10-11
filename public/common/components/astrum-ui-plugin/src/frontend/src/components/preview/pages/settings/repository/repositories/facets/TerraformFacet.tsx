import React from 'react';

import {
  SettingsFormSection,
  SettingsTextArea,
  SettingsPasswordInput,
} from '../../../../../shared/form';

import { RepositoryFormData } from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

interface TerraformFacetProps {
  formData: RepositoryFormData;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  errors?: Record<string, string | undefined>;
  isEdit: boolean;
}

/**
 * TerraformFacet - signing of a Terraform hosted repository (`attributes.terraformSigning`).
 *
 * The passphrase is a secret: the form never receives the saved one and only sends what the admin
 * types, so the field starts empty when a repository is edited.
 */
export function TerraformFacet({ formData, onNestedChange, errors, isEdit }: TerraformFacetProps) {
  const strings = UIStrings.TERRAFORM.SIGNING;

  return (
    <SettingsFormSection title={strings.title} description={strings.description}>
      <SettingsTextArea
        name="terraform-keypair"
        label={strings.KEYPAIR.label}
        value={formData.terraformSigning?.keypair || ''}
        onChange={(value: string) => onNestedChange('terraformSigning', { keypair: value })}
        helpText={strings.KEYPAIR.helpText}
        error={errors?.['terraformSigning.keypair']}
        rows={8}
        monospace
      />

      <SettingsPasswordInput
        name="terraform-passphrase"
        label={strings.PASSPHRASE.label}
        value={formData.terraformSigning?.passphrase || ''}
        onChange={(value: string) => onNestedChange('terraformSigning', { passphrase: value })}
        helpText={isEdit ? strings.PASSPHRASE.EDIT_HELP_TEXT : strings.PASSPHRASE.helpText}
        autoComplete="new-password"
        showToggle={false}
      />
    </SettingsFormSection>
  );
}

export default TerraformFacet;
