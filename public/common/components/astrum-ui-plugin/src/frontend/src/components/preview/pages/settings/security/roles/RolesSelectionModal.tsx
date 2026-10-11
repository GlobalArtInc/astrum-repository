

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { Box, Flex, Text, Heading } from '@radix-ui/themes';
import { X, Loader2 } from 'lucide-react';

import { SettingsButton, SettingsTransferList } from '../../../../shared/form';
import { RolesSelectionModalProps } from './types';

import './RolesSelectionModal.scss';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * RolesSelectionModal - Modal for selecting roles to nest within another role
 */
export function RolesSelectionModal({
  isOpen,
  onClose,
  availableRoles,
  selectedRoles,
  currentRoleId,
  onSave,
  loading = false,
}: RolesSelectionModalProps) {
  const [selection, setSelection] = useState<string[]>(selectedRoles);

  // Reset selection when modal opens
  useEffect(() => {
    if (isOpen) {
      setSelection(selectedRoles);
    }
  }, [isOpen, selectedRoles]);

  // Filter out current role to prevent circular reference
  const filteredAvailableRoles = useMemo(() => {
    return availableRoles.filter((r) => r.id !== currentRoleId);
  }, [availableRoles, currentRoleId]);

  // Convert to transfer list format
  const availableItems = useMemo(() => {
    return filteredAvailableRoles.map((r) => ({ id: r.id, name: r.name }));
  }, [filteredAvailableRoles]);

  const selectedItems = useMemo(() => {
    return filteredAvailableRoles
      .filter((r) => selection.includes(r.id))
      .map((r) => ({ id: r.id, name: r.name }));
  }, [filteredAvailableRoles, selection]);

  const handleChange = useCallback((newSelected: Array<{ id: string; name: string }>) => {
    setSelection(newSelected.map((r) => r.id));
  }, []);

  const handleSave = useCallback(() => {
    onSave(selection);
    onClose();
  }, [selection, onSave, onClose]);

  const handleCancel = useCallback(() => {
    setSelection(selectedRoles);
    onClose();
  }, [selectedRoles, onClose]);

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && handleCancel()}>
      <Dialog.Portal>
        <Dialog.Overlay className="roles-modal__overlay" />
        <Dialog.Content className="roles-modal__content">
          <Flex justify="between" align="center" className="roles-modal__header">
            <Dialog.Title asChild>
              <Heading size="5">{RolesStrings.ROLES_MODAL.SELECT_CONTAINED_ROLES}</Heading>
            </Dialog.Title>
            <Dialog.Close asChild>
              <button type="button" className="roles-modal__close" aria-label={CommonStrings.ACTIONS.CLOSE}>
                <X size={18} />
              </button>
            </Dialog.Close>
          </Flex>

          <Dialog.Description className="roles-modal__description">
            <Text size="2">
              {RolesStrings.ROLES_MODAL.SELECT_ROLES_CONTAIN_WITHIN_ROLE}
            </Text>
          </Dialog.Description>

          <Box className="roles-modal__body">
            {loading ? (
              <Flex align="center" justify="center" className="roles-modal__loading">
                <Loader2 size={24} className="roles-modal__spinner" />
                <Text size="2">{RolesStrings.ROLES_MODAL.LOADING_ROLES}</Text>
              </Flex>
            ) : (
              <SettingsTransferList
                name="roles-selection"
                availableItems={availableItems}
                selectedItems={selectedItems}
                onChange={handleChange}
                availableLabel={RolesStrings.ROLES_MODAL.AVAILABLE_ROLES}
                selectedLabel={RolesStrings.ROLES_MODAL.CONTAINED_ROLES}
                getItemId={(item) => item.id}
                getItemLabel={(item) => item.name}
              />
            )}
          </Box>

          <Flex justify="end" gap="3" className="roles-modal__footer">
            <SettingsButton variant="secondary" onClick={handleCancel}>
              {CommonStrings.ACTIONS.CANCEL}
            </SettingsButton>
            <SettingsButton variant="primary" onClick={handleSave} disabled={loading}>
              {RolesStrings.ROLES_MODAL.APPLY_SELECTION}
            </SettingsButton>
          </Flex>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export default RolesSelectionModal;


