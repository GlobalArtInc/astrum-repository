

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { Box, Flex, Text, Heading } from '@radix-ui/themes';
import { X, Loader2 } from 'lucide-react';

import { SettingsButton, SettingsTransferList } from '../../../../shared/form';
import { PrivilegesSelectionModalProps } from './types';

import './PrivilegesSelectionModal.scss';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * PrivilegesSelectionModal - Modal for selecting privileges to assign to a role
 */
export function PrivilegesSelectionModal({
  isOpen,
  onClose,
  availablePrivileges,
  selectedPrivileges,
  onSave,
  loading = false,
}: PrivilegesSelectionModalProps) {
  const [selection, setSelection] = useState<string[]>(selectedPrivileges);

  // Reset selection when modal opens
  useEffect(() => {
    if (isOpen) {
      setSelection(selectedPrivileges);
    }
  }, [isOpen, selectedPrivileges]);

  // Convert to transfer list format
  const availableItems = useMemo(() => {
    return availablePrivileges.map((p) => ({ id: p.id, name: p.name }));
  }, [availablePrivileges]);

  const selectedItems = useMemo(() => {
    return availablePrivileges
      .filter((p) => selection.includes(p.id))
      .map((p) => ({ id: p.id, name: p.name }));
  }, [availablePrivileges, selection]);

  const handleChange = useCallback((newSelected: Array<{ id: string; name: string }>) => {
    setSelection(newSelected.map((p) => p.id));
  }, []);

  const handleSave = useCallback(() => {
    onSave(selection);
    onClose();
  }, [selection, onSave, onClose]);

  const handleCancel = useCallback(() => {
    setSelection(selectedPrivileges);
    onClose();
  }, [selectedPrivileges, onClose]);

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && handleCancel()}>
      <Dialog.Portal>
        <Dialog.Overlay className="privileges-modal__overlay" />
        <Dialog.Content className="privileges-modal__content">
          <Flex justify="between" align="center" className="privileges-modal__header">
            <Dialog.Title asChild>
              <Heading size="5">{RolesStrings.PRIVILEGES_MODAL.SELECT_PRIVILEGES}</Heading>
            </Dialog.Title>
            <Dialog.Close asChild>
              <button type="button" className="privileges-modal__close" aria-label={CommonStrings.ACTIONS.CLOSE}>
                <X size={18} />
              </button>
            </Dialog.Close>
          </Flex>

          <Dialog.Description className="privileges-modal__description">
            <Text size="2">
              {RolesStrings.PRIVILEGES_MODAL.SELECT_PRIVILEGES_GRANT_ROLE_DOUBLE}
            </Text>
          </Dialog.Description>

          <Box className="privileges-modal__body">
            {loading ? (
              <Flex align="center" justify="center" className="privileges-modal__loading">
                <Loader2 size={24} className="privileges-modal__spinner" />
                <Text size="2">{RolesStrings.PRIVILEGES_MODAL.LOADING_PRIVILEGES}</Text>
              </Flex>
            ) : (
              <SettingsTransferList
                name="privileges-selection"
                availableItems={availableItems}
                selectedItems={selectedItems}
                onChange={handleChange}
                availableLabel={RolesStrings.PRIVILEGES_MODAL.AVAILABLE_PRIVILEGES}
                selectedLabel={RolesStrings.PRIVILEGES_MODAL.SELECTED_PRIVILEGES}
                getItemId={(item) => item.id}
                getItemLabel={(item) => item.name}
              />
            )}
          </Box>

          <Flex justify="end" gap="3" className="privileges-modal__footer">
            <SettingsButton variant="secondary" onClick={handleCancel}>
              {CommonStrings.ACTIONS.CANCEL}
            </SettingsButton>
            <SettingsButton variant="primary" onClick={handleSave} disabled={loading}>
              {RolesStrings.PRIVILEGES_MODAL.APPLY_SELECTION}
            </SettingsButton>
          </Flex>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export default PrivilegesSelectionModal;


