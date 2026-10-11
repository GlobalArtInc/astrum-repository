

import React, { useState, useRef, useEffect } from 'react';
import { AlertDialog, Box, Flex, Text, Button, TextField, IconButton } from '@radix-ui/themes';
import { X } from 'lucide-react';

import type { DangerousFieldInfo } from './dangerousFields';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export interface DangerousEditConfirmDialogProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  blobStoreName: string;
  changedFields: DangerousFieldInfo[];
}

export function DangerousEditConfirmDialog({
  open,
  onClose,
  onConfirm,
  blobStoreName,
  changedFields,
}: DangerousEditConfirmDialogProps) {
  const [confirmationText, setConfirmationText] = useState('');
  const [showError, setShowError] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const isValid = confirmationText === blobStoreName;

  useEffect(() => {
    if (open && inputRef.current) {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          inputRef.current?.focus();
        });
      });
    }
    if (!open) {
      setConfirmationText('');
      setShowError(false);
    }
  }, [open]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setConfirmationText(value);
    setShowError(value.length > 0 && value !== blobStoreName);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && isValid) {
      onConfirm();
    }
  };

  const handleConfirm = () => {
    if (isValid) {
      onConfirm();
    }
  };

  return (
    <AlertDialog.Root open={open} onOpenChange={onClose}>
      <AlertDialog.Content maxWidth="450px">
        <Flex align="start" justify="between" mb="2">
          <AlertDialog.Title size="5">{BlobStoresStrings.DANGEROUS_EDIT.UPDATE_BLOB_STORE}</AlertDialog.Title>
          <IconButton
            variant="ghost"
            size="1"
            color="gray"
            aria-label={CommonStrings.ACTIONS.CLOSE}
            onClick={onClose}
          >
            <X size={16} />
          </IconButton>
        </Flex>

        <AlertDialog.Description size="2" mb="4">
          {BlobStoresStrings.DANGEROUS_EDIT.YOU_ABOUT_CHANGE_CONFIGURATION_FIELDS}
        </AlertDialog.Description>

        <Box
          p="3"
          mb="4"
          style={{
            backgroundColor: 'var(--amber-2)',
            border: '1px solid var(--amber-6)',
            borderRadius: '6px',
          }}
        >
          <Text size="2" weight="medium" style={{ display: 'block' }} mb="2">
            {BlobStoresStrings.DANGEROUS_EDIT.FOLLOWING_FIELDS_HAVE_BEEN_MODIFIED}
          </Text>
          <ul style={{ margin: 0, paddingLeft: '20px' }}>
            {changedFields.map(({ field, label }) => (
              <li key={field}>
                <Text size="2">{label}</Text>
              </li>
            ))}
          </ul>
        </Box>

        <Box
          p="3"
          mb="4"
          style={{
            backgroundColor: 'var(--amber-2)',
            border: '1px solid var(--amber-6)',
            borderRadius: '6px',
          }}
        >
          <Text size="2" weight="medium" style={{ display: 'block' }}>
            {blobStoreName}
          </Text>
          <Text size="1" color="gray">
            {BlobStoresStrings.DANGEROUS_EDIT.TYPE_NAME_CONFIRM}
          </Text>
        </Box>

        <Box mb="4">
          <Flex align="center" gap="1" mb="2">
            <Text size="2" weight="bold">
              {BlobStoresStrings.DANGEROUS_EDIT.ACKNOWLEDGEMENT}
            </Text>
            <Text size="2" style={{ color: 'var(--amber-11)' }}>
              *
            </Text>
          </Flex>
          <TextField.Root
            ref={inputRef}
            size="2"
            value={confirmationText}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            placeholder={BlobStoresStrings.DANGEROUS_EDIT.TYPE_CONFIRM(blobStoreName)}
            color={showError ? 'red' : undefined}
            style={
              showError
                ? {
                    outline: '2px solid var(--red-9)',
                    outlineOffset: '-2px',
                  }
                : undefined
            }
            aria-label={BlobStoresStrings.DANGEROUS_EDIT.ACKNOWLEDGEMENT_ARIA}
            autoFocus
          />
          {showError && (
            <Text
              size="1"
              color="red"
              mt="1"
              style={{ display: 'block' }}
              role="alert"
            >
              {BlobStoresStrings.DANGEROUS_EDIT.CONFIRMATION_TEXT_PROVIDED_INCORRECT}
            </Text>
          )}
        </Box>

        <Flex gap="3" mt="4" justify="end">
          <AlertDialog.Cancel>
            <Button variant="surface" size="2" onClick={onClose}>
              {CommonStrings.ACTIONS.CANCEL}
            </Button>
          </AlertDialog.Cancel>
          <AlertDialog.Action>
            <Button
              variant="solid"
              color="amber"
              size="2"
              onClick={handleConfirm}
              disabled={!isValid}
            >
              {BlobStoresStrings.DANGEROUS_EDIT.CONFIRM_UPDATE}
            </Button>
          </AlertDialog.Action>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}

export default DangerousEditConfirmDialog;
