

import React, { useState } from 'react';
import { AlertTriangle } from 'lucide-react';
import {
  SettingsTextInput,
  SettingsAlert,
  SettingsButton
} from '../../../../shared/form';
import './ConvertToGroupModal.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';
import CommonStrings from '../../../../constants/CommonStrings';

interface ConvertToGroupModalProps {
  blobStoreName: string;
  onConfirm: (newGroupName: string) => void;
  onCancel: () => void;
  promoting?: boolean;
}

const STRINGS = {
  HEADER: BlobStoresStrings.CONVERT_TO_GROUP.CONVERT_GROUP_BLOB_STORE,
  LABEL: BlobStoresStrings.CONVERT_TO_GROUP.RENAME_ORIGINAL_BLOB_STORE,
  SUBLABEL: BlobStoresStrings.CONVERT_TO_GROUP.ASSIGN_NEW_NAME_ORIGINAL_BLOB,
  ALERT: BlobStoresStrings.CONVERT_TO_GROUP.YOU_CONVERTING_GROUP_BLOB_STORE,
  CONVERT_BUTTON: BlobStoresStrings.CONVERT_TO_GROUP.CONVERT,
  CANCEL_BUTTON: CommonStrings.ACTIONS.CANCEL,
  PLACEHOLDER: BlobStoresStrings.CONVERT_TO_GROUP.ENTER_NEW_NAME_ORIGINAL_BLOB
};

export default function ConvertToGroupModal({
  blobStoreName,
  onConfirm,
  onCancel,
  promoting = false
}: ConvertToGroupModalProps) {
  const [newName, setNewName] = useState(`${blobStoreName}-original`);
  const [error, setError] = useState<string | null>(null);

  const handleConfirm = () => {
    if (!newName.trim()) {
      setError(CommonStrings.MESSAGES.NAME_REQUIRED);
      return;
    }

    if (!/^[a-zA-Z0-9_-]+$/.test(newName)) {
      setError(BlobStoresStrings.CONVERT_TO_GROUP.NAME_CAN_ONLY_CONTAIN_LETTERS);
      return;
    }

    onConfirm(newName);
  };

  return (
    <div className="convert-to-group-modal__overlay">
      <div className="convert-to-group-modal">
        <h2 className="convert-to-group-modal__header">{STRINGS.HEADER}</h2>

        <SettingsAlert variant="warning" icon={<AlertTriangle size={16} />}>
          {STRINGS.ALERT}
        </SettingsAlert>

        <div className="convert-to-group-modal__content">
          <SettingsTextInput
            label={STRINGS.LABEL}
            helpText={STRINGS.SUBLABEL}
            value={newName}
            onChange={(value) => {
              setNewName(value);
              setError(null);
            }}
            placeholder={STRINGS.PLACEHOLDER}
            error={error || undefined}
            required
            disabled={promoting}
          />
        </div>

        <div className="convert-to-group-modal__actions">
          <SettingsButton
            variant="ghost"
            onClick={onCancel}
            disabled={promoting}
          >
            {STRINGS.CANCEL_BUTTON}
          </SettingsButton>
          <SettingsButton
            variant="primary"
            onClick={handleConfirm}
            disabled={promoting || !newName.trim()}
          >
            {promoting ? BlobStoresStrings.CONVERT_TO_GROUP.CONVERTING : STRINGS.CONVERT_BUTTON}
          </SettingsButton>
        </div>
      </div>
    </div>
  );
}

