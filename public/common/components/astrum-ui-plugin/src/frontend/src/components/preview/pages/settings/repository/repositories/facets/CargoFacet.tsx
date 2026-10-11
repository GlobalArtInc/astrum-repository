import React from 'react';

import { SettingsFormSection, SettingsCheckbox } from '../../../../../shared/form';

import { RepositoryFormData } from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

interface CargoFacetProps {
  formData: RepositoryFormData;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
}

/**
 * CargoFacet - Cargo repository configuration (`attributes.cargo`), for hosted, proxy and group
 * repositories alike: the server applies the setting to every Cargo repository.
 *
 * Fields:
 * - Require authentication: anonymous users may not read the repository even when anonymous
 *   access is enabled on the instance
 */
export function CargoFacet({ formData, onNestedChange }: CargoFacetProps) {
  return (
    <SettingsFormSection
      title={UIStrings.CARGO.SECTION.title}
      description={UIStrings.CARGO.SECTION.description}
    >
      <SettingsCheckbox
        name="cargo-requireAuthentication"
        label={UIStrings.CARGO.REQUIRE_AUTHENTICATION.label}
        checked={formData.cargo?.requireAuthentication ?? false}
        onChange={(checked) => onNestedChange('cargo', { requireAuthentication: checked })}
        description={UIStrings.CARGO.REQUIRE_AUTHENTICATION.description}
        analyticsId="as-repository-cargo-require-authentication"
      />
    </SettingsFormSection>
  );
}

export default CargoFacet;
