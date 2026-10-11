

import React from 'react';
import { Folder, } from 'lucide-react';
import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsAlert
} from '../../../../shared/form';
import { useIsClustered } from '../../../../shared/hooks';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { shouldShowHaPathWarning, HA_PATH_WARNING } from './haPathWarning';
import type { BlobStoreFormData } from './types';
import './FileBlobStoreSettings.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';

interface FileBlobStoreSettingsProps {
  data: BlobStoreFormData;
  onChange: (path: string, value: unknown) => void;
  disabled?: boolean;
  isEdit?: boolean;
  errors?: Record<string, string | null>;
}

const STRINGS = {
  TITLE: BlobStoresStrings.FILE.FILE_STORAGE_CONFIGURATION,
  DESCRIPTION: BlobStoresStrings.FILE.CONFIGURE_LOCAL_FILESYSTEM_PATH_BLOB,
  PATH: {
    label: BlobStoresStrings.FILE.PATH,
    helpText: BlobStoresStrings.FILE.ABSOLUTE_PATH_PATH_RELATIVE_DATA,
    placeholder: '/path/to/blob/storage'
  },
  WARNING: BlobStoresStrings.FILE.CHANGING_PATH_NOT_MIGRATE_EXISTING
};

export default function FileBlobStoreSettings({
  data,
  onChange,
  disabled = false,
  isEdit = false,
  errors = {},
}: FileBlobStoreSettingsProps) {
  const isClustered = useIsClustered();
  const workDirectory = ExtJS.state()?.getValue?.('astrum.application.workDirectory') || '';
  const showHaWarning = isClustered && shouldShowHaPathWarning(data.path, workDirectory);

  return (
    <div className="file-blob-store-settings">
      <SettingsFormSection
        title={STRINGS.TITLE}
        description={STRINGS.DESCRIPTION}
        icon={<Folder size={20} />}
      >
        {isEdit && (
          <SettingsAlert type="warning">
            {STRINGS.WARNING}
          </SettingsAlert>
        )}

        {showHaWarning && (
          <SettingsAlert type="warning">
            <strong>{HA_PATH_WARNING.TITLE}</strong>
            <br />
            {HA_PATH_WARNING.MESSAGE}
          </SettingsAlert>
        )}

        <SettingsTextInput
          name="file-path"
          label={STRINGS.PATH.label}
          value={data.path || ''}
          onChange={(value) => onChange('path', value)}
          helpText={STRINGS.PATH.helpText}
          placeholder={STRINGS.PATH.placeholder}
          error={errors.path ?? undefined}
          required
          disabled={disabled}
          monospace
        />
      </SettingsFormSection>
    </div>
  );
}
