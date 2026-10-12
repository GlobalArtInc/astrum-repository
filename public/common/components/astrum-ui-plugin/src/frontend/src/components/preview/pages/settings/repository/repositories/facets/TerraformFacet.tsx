import React from 'react';

import {
  SettingsCheckbox,
  SettingsFormSection,
  SettingsTextArea,
  SettingsTextInput,
  SettingsPasswordInput,
} from '../../../../../shared/form';

import { RepositoryFormData, RepositoryType } from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';
import CommonStrings from '../../../../../constants/CommonStrings';

interface TerraformFacetProps {
  formData: RepositoryFormData;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  errors?: Record<string, string | undefined>;
  isEdit: boolean;
  repoType: RepositoryType;
}

/**
 * TerraformFacet - Terraform repository configuration (`attributes.terraform`) for hosted, proxy and group
 * repositories, and the signing key of a hosted repository (`attributes.terraformSigning`).
 *
 * Fields:
 * - Registry host: the host name Terraform configurations address this repository by; the server then answers
 *   service discovery at the root of that host with this repository
 * - Upstream host (proxy only): the host name the providers of the remote registry carry in their addresses,
 *   which the provider network mirror protocol names
 * - Require authentication
 * - Signing key and passphrase (hosted only). The passphrase is a secret: the form never receives the saved one
 *   and only sends what the admin types, so the field starts empty when a repository is edited.
 */
export function TerraformFacet({ formData, onNestedChange, errors, isEdit, repoType }: TerraformFacetProps) {
  const settings = UIStrings.TERRAFORM.SETTINGS;
  const signing = UIStrings.TERRAFORM.SIGNING;

  return (
    <>
      <SettingsFormSection title={settings.title} description={settings.description}>
        <SettingsTextInput
          name="terraform-registryHost"
          label={settings.REGISTRY_HOST.label}
          value={formData.terraform?.registryHost || ''}
          onChange={(value: string) => onNestedChange('terraform', { registryHost: value || null })}
          helpText={settings.REGISTRY_HOST.helpText}
          placeholder={CommonStrings.MESSAGES.EXAMPLE('terraform.example.com')}
          error={errors?.['terraform.registryHost']}
        />

        {repoType === 'proxy' && (
          <SettingsTextInput
            name="terraform-upstreamHost"
            label={settings.UPSTREAM_HOST.label}
            value={formData.terraform?.upstreamHost || ''}
            onChange={(value: string) => onNestedChange('terraform', { upstreamHost: value || null })}
            helpText={settings.UPSTREAM_HOST.helpText}
            placeholder={CommonStrings.MESSAGES.EXAMPLE('registry.terraform.io')}
            error={errors?.['terraform.upstreamHost']}
          />
        )}

        <SettingsCheckbox
          name="terraform-requireAuthentication"
          label={settings.REQUIRE_AUTHENTICATION.label}
          checked={formData.terraform?.requireAuthentication ?? false}
          onChange={(checked: boolean) => onNestedChange('terraform', { requireAuthentication: checked })}
          description={settings.REQUIRE_AUTHENTICATION.description}
          analyticsId="as-repository-terraform-require-authentication"
        />
      </SettingsFormSection>

      {repoType === 'hosted' && (
        <SettingsFormSection title={signing.title} description={signing.description}>
          <SettingsTextArea
            name="terraform-keypair"
            label={signing.KEYPAIR.label}
            value={formData.terraformSigning?.keypair || ''}
            onChange={(value: string) => onNestedChange('terraformSigning', { keypair: value })}
            helpText={signing.KEYPAIR.helpText}
            error={errors?.['terraformSigning.keypair']}
            rows={8}
            monospace
          />

          <SettingsPasswordInput
            name="terraform-passphrase"
            label={signing.PASSPHRASE.label}
            value={formData.terraformSigning?.passphrase || ''}
            onChange={(value: string) => onNestedChange('terraformSigning', { passphrase: value })}
            helpText={isEdit ? signing.PASSPHRASE.EDIT_HELP_TEXT : signing.PASSPHRASE.helpText}
            autoComplete="new-password"
            showToggle={false}
          />
        </SettingsFormSection>
      )}
    </>
  );
}

export default TerraformFacet;
