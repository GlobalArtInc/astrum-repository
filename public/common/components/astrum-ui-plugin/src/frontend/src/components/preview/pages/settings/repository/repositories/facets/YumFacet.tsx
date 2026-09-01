

import React from 'react';

import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsTextArea,
  SettingsSelect,
  SettingsPasswordInput,
} from '../../../../../shared/form';

import { RepositoryFormData, RepositoryFormErrors } from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

interface YumFacetProps {
  formData: RepositoryFormData;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  errors?: RepositoryFormErrors;
  repoType: 'hosted' | 'proxy' | 'group';
}

const DEPLOY_POLICY_OPTIONS = [
  { value: 'STRICT', label: UIStrings.YUM.HOSTED.DEPLOY_POLICY.STRICT },
  { value: 'PERMISSIVE', label: UIStrings.YUM.HOSTED.DEPLOY_POLICY.PERMISSIVE },
];

/**
 * YumFacet - Yum/RPM-specific repository configuration
 *
 * Fields:
 * - Repodata Depth (0-5)
 * - Deploy Policy (Strict/Permissive) for hosted repos
 * - GPG Signing keypair and passphrase (proxy and group)
 */
export function YumFacet({
  formData,
  onNestedChange,
  errors,
  repoType,
}: YumFacetProps) {
  const isHosted = repoType === 'hosted';
  const showSigning = repoType === 'proxy' || repoType === 'group';

  return (
    <>
      {showSigning && (
        <SettingsFormSection
          title={UIStrings.YUM.SIGNING.title}
          description={UIStrings.YUM.SIGNING.description}
        >
          <SettingsTextArea
            name="yumSigning-keypair"
            label={UIStrings.YUM.SIGNING.KEYPAIR.label}
            value={formData.yumSigning?.keypair || ''}
            onChange={(value) => onNestedChange('yumSigning', { keypair: value })}
            helpText={UIStrings.YUM.SIGNING.KEYPAIR.helpText}
            rows={8}
            monospace
          />

          <SettingsPasswordInput
            name="yumSigning-passphrase"
            label={UIStrings.YUM.SIGNING.PASSPHRASE.label}
            value={formData.yumSigning?.passphrase || ''}
            onChange={(value) => onNestedChange('yumSigning', { passphrase: value })}
            helpText={UIStrings.YUM.SIGNING.PASSPHRASE.helpText}
            showToggle={false}
          />
        </SettingsFormSection>
      )}

      {isHosted && (
        <SettingsFormSection
          title={UIStrings.YUM.HOSTED.title}
          description={UIStrings.YUM.HOSTED.description}
        >
          <SettingsTextInput
            name="yum-repodataDepth"
            label={UIStrings.YUM.HOSTED.REPODATA_DEPTH.label}
            value={formData.yum?.repodataDepth?.toString() || '0'}
            onChange={(value) => {
              const parsed = parseInt(value, 10);
              const clamped = Number.isNaN(parsed) ? 0 : Math.max(0, Math.min(5, parsed));
              onNestedChange('yum', { repodataDepth: clamped });
            }}
            helpText={UIStrings.YUM.HOSTED.REPODATA_DEPTH.helpText}
            type="number"
            min={0}
            max={5}
            required
          />

          <SettingsSelect
            name="yum-deployPolicy"
            label={UIStrings.YUM.HOSTED.DEPLOY_POLICY.label}
            value={formData.yum?.deployPolicy || 'STRICT'}
            onChange={(value) => onNestedChange('yum', { deployPolicy: value as 'STRICT' | 'PERMISSIVE' })}
            options={DEPLOY_POLICY_OPTIONS}
            helpText={UIStrings.YUM.HOSTED.DEPLOY_POLICY.helpText}
          />
        </SettingsFormSection>
      )}
    </>
  );
}

export default YumFacet;
