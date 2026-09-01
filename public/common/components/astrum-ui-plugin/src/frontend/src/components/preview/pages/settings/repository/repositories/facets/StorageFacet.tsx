

import React from 'react';

import {
  SettingsFormSection,
  SettingsSelect,
  SettingsCheckbox,
} from '../../../../../shared/form';

import {
  RepositoryFormData,
  RepositoryFormErrors,
  BlobStore,
} from '../types';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

interface StorageFacetProps {
  formData: RepositoryFormData;
  onChange: (updates: Partial<RepositoryFormData>) => void;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  errors?: RepositoryFormErrors;
  isEdit?: boolean;
  isCloud?: boolean;
  blobStores: BlobStore[];
}

/**
 * StorageFacet - Blob store and content validation settings
 */
export function StorageFacet({
  formData,
  onChange,
  onNestedChange,
  errors,
  isEdit,
  isCloud = false,
  blobStores,
}: StorageFacetProps) {
  const handleBlobStoreChange = (value: string) => {
    onNestedChange('storage', { blobStoreName: value });
  };

  const handleContentValidationChange = (checked: boolean) => {
    onNestedChange('storage', { strictContentTypeValidation: checked });
  };

  const blobStoreOptions = blobStores.map((store) => ({
    value: store.name,
    label: store.name,
  }));

  return (
    <SettingsFormSection title={UIStrings.STORAGE.SECTION.title}>
      {!isCloud && (
        <SettingsSelect
          name="storage-blobStoreName"
          label={UIStrings.STORAGE.BLOB_STORE.label}
          value={formData.storage?.blobStoreName || ''}
          onChange={handleBlobStoreChange}
          options={[
            { value: '', label: UIStrings.STORAGE.BLOB_STORE.selectPlaceholder },
            ...blobStoreOptions,
          ]}
          error={errors?.storage?.blobStoreName}
          disabled={isEdit}
          required
          helpText={isEdit ? UIStrings.STORAGE.BLOB_STORE.editHelpText : UIStrings.STORAGE.BLOB_STORE.helpText}
        />
      )}

      <SettingsCheckbox
        name="storage-strictContentTypeValidation"
        label={UIStrings.STORAGE.STRICT_CONTENT_VALIDATION.label}
        checked={formData.storage?.strictContentTypeValidation ?? true}
        onChange={handleContentValidationChange}
        description={UIStrings.STORAGE.STRICT_CONTENT_VALIDATION.description}
      />
    </SettingsFormSection>
  );
}

export default StorageFacet;

