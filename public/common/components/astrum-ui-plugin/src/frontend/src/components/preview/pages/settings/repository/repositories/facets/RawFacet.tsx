

import React from 'react';
import { SettingsAlert, SettingsFormSection, SettingsSelect } from '../../../../../shared/form';
import { RepositoryFormData } from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';
import { ExtJS } from '@globalart/astrum-ui-plugin';

interface RawFacetProps {
  formData: RepositoryFormData;
  onNestedChange: <K extends keyof RepositoryFormData>(key: K, updates: Partial<RepositoryFormData[K]>) => void;
}

const CONTENT_DISPOSITION_OPTIONS = [
  { value: 'INLINE', label: UIStrings.RAW.CONTENT_DISPOSITION.INLINE },
  { value: 'ATTACHMENT', label: UIStrings.RAW.CONTENT_DISPOSITION.ATTACHMENT },
];

/**
 * RawFacet - Raw format content disposition setting
 */
export function RawFacet({ formData, onNestedChange }: RawFacetProps) {
  const isCloud = ExtJS.useState?.(() => ExtJS.state()?.getValue?.('isCloud'));

  if (isCloud) {
    return null;
  }

  return (
    <SettingsFormSection title={UIStrings.RAW.SECTION.title} description={UIStrings.RAW.SECTION.description}>
      <SettingsSelect
        name="raw-contentDisposition"
        label={UIStrings.RAW.CONTENT_DISPOSITION.label}
        value={formData.raw?.contentDisposition || 'ATTACHMENT'}
        onChange={(value) => onNestedChange('raw', { contentDisposition: value as 'INLINE' | 'ATTACHMENT' })}
        options={CONTENT_DISPOSITION_OPTIONS}
        helpText={UIStrings.RAW.CONTENT_DISPOSITION.helpText}
      />

      {(formData.raw?.contentDisposition || 'ATTACHMENT') === 'INLINE' && (
        <SettingsAlert type="warning">
          {UIStrings.RAW.CONTENT_DISPOSITION.inlineWarning}
        </SettingsAlert>
      )}
    </SettingsFormSection>
  );
}

export default RawFacet;
