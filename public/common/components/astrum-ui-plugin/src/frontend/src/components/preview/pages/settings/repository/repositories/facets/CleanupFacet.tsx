

import React, { useMemo } from 'react';
import { Box, Text } from '@radix-ui/themes';
import { ExternalLink } from 'lucide-react';

import {
  SettingsFormSection,
  SettingsTransferList,
} from '../../../../../shared/form';

import {
  RepositoryFormData,
  RepositoryFormErrors,
  CleanupPolicy,
} from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

interface CleanupFacetProps {
  formData: RepositoryFormData;
  onChange: (updates: Partial<RepositoryFormData>) => void;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  errors?: RepositoryFormErrors;
  cleanupPolicies: CleanupPolicy[];
}

/**
 * CleanupFacet - Cleanup policy selection for repositories using transfer list
 */
export function CleanupFacet({
  formData,
  onChange,
  onNestedChange,
  errors,
  cleanupPolicies,
}: CleanupFacetProps) {
  const selectedPolicyNames = formData.cleanup?.policyNames || [];

  // Filter to policies matching this repository's format, plus the
  // "all formats" sentinel '*' which the REST API uses to represent
  // policies stored with format = 'ALL_FORMATS' (see
  // CleanupPolicyXO.fromCleanupPolicy on the backend).
  const repoFormat = formData.format || '';
  const availablePolicies = useMemo(() => {
    return cleanupPolicies.filter(
      (policy) => policy.format === repoFormat || policy.format === '*'
    );
  }, [cleanupPolicies, repoFormat]);

  // Convert selected policy names to full policy objects
  const selectedPolicies = useMemo(() => {
    return selectedPolicyNames
      .map((name) => cleanupPolicies.find((p) => p.name === name))
      .filter((policy): policy is CleanupPolicy => policy !== undefined);
  }, [selectedPolicyNames, cleanupPolicies]);

  const handlePoliciesChange = (newSelectedPolicies: CleanupPolicy[]) => {
    const policyNames = newSelectedPolicies.map((p) => p.name);

    if (policyNames.length === 0) {
      onChange({ cleanup: null });
    } else {
      onNestedChange('cleanup', { policyNames });
    }
  };

  return (
    <SettingsFormSection title={UIStrings.CLEANUP.SECTION.title}>
      <Box className="cleanup-facet">
        <Text size="2" className="cleanup-facet__help">
          {UIStrings.CLEANUP.helpText}
          {' '}
          <a
            href="#preview/admin/repository/cleanup-policies"
            className="cleanup-facet__link"
          >
            {UIStrings.CLEANUP.MANAGE_LINK.label}
            <ExternalLink size={12} />
          </a>
        </Text>

        <SettingsTransferList
          name="cleanup-policies"
          label={UIStrings.CLEANUP.TRANSFER_LIST.label}
          availableItems={availablePolicies}
          selectedItems={selectedPolicies}
          onChange={handlePoliciesChange}
          availableLabel={UIStrings.CLEANUP.TRANSFER_LIST.availableLabel}
          selectedLabel={UIStrings.CLEANUP.TRANSFER_LIST.selectedLabel}
          getItemId={(policy) => policy.name}
          getItemLabel={(policy) => policy.name}
          helpText={UIStrings.CLEANUP.TRANSFER_LIST.helpText}
        />
      </Box>
    </SettingsFormSection>
  );
}

export default CleanupFacet;

