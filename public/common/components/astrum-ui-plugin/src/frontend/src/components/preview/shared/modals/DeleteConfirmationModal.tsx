

import React, { useState, useRef, useEffect } from 'react';
import { AlertDialog, Box, Flex, Text, Button, TextField, IconButton } from '@radix-ui/themes';
import { X } from 'lucide-react';
import CommonStrings from '../../constants/CommonStrings';
import SharedStrings from '../../constants/SharedStrings';

// Fixed acknowledgement word required to confirm any destructive deletion. Using a
// single literal word — instead of per-entity names — keeps the friction predictable
// and short to type while still forcing a deliberate action. Displayed as-is; the
// match is case-insensitive so "delete", "Delete", "DELETE" all confirm.
const CONFIRMATION_TEXT = SharedStrings.DELETE_CONFIRMATION.CONFIRMATION_WORD;

const ENTITY_KEYS: Record<string, string> = {
  'blob store': 'BLOB_STORE',
  repository: 'REPOSITORY',
  'routing rule': 'ROUTING_RULE',
  task: 'TASK',
  user: 'USER',
};

function entityStringsOf(entityType: string): { TITLE: string; MESSAGE: string } | undefined {
  const key = ENTITY_KEYS[entityType];
  return key ? SharedStrings.DELETE_CONFIRMATION.ENTITIES[key] : undefined;
}

export interface DeleteConfirmationModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  entityName?: string;
  entityType: string;
  loading?: boolean;
  /** Optional extra content (e.g. resource details) rendered above the
   *  acknowledgement input. Lets callers surface context like task type or
   *  schedule without forking the modal. */
  children?: React.ReactNode;
}

/**
 * Reusable delete confirmation modal with text verification.
 *
 * Follows astrumone design system standards:
 * - Modal width: 450px (destructive confirmation)
 * - Requires typing "Delete" (case-insensitive) for all deletions
 * - Entity name is displayed in the description but not required for verification
 *
 * @param open - Controls modal visibility
 * @param onClose - Called when modal should close (cancel, X button)
 * @param onConfirm - Called when user confirms deletion with valid text
 * @param entityName - If provided, displayed in description for context
 * @param entityType - The entity type in English (e.g., "repository", "user", "blob store"); the title and
 *   message of the known types are translated, other types are shown as given
 * @param loading - Shows loading state during delete operation
 */
export function DeleteConfirmationModal({
  open,
  onClose,
  onConfirm,
  entityName,
  entityType,
  loading = false,
  children,
}: DeleteConfirmationModalProps) {
  const [confirmationText, setConfirmationText] = useState('');
  const [showError, setShowError] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // Acknowledgement is a case-insensitive match against the literal CONFIRMATION_TEXT
  // in every case. The entity name still flows through to the description so the user
  // can see what they're about to delete, but they no longer have to type it.
  const expectedLower = CONFIRMATION_TEXT.toLowerCase();
  const isValid = confirmationText.toLowerCase() === expectedLower;
  const entityStrings = entityStringsOf(entityType);

  // Focus input when modal opens
  useEffect(() => {
    if (open && inputRef.current) {
      // Use requestAnimationFrame twice to ensure DOM has settled
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          inputRef.current?.focus();
        });
      });
    }
    // Reset state when modal closes
    if (!open) {
      setConfirmationText('');
      setShowError(false);
    }
  }, [open]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setConfirmationText(value);
    // Show the inline error only when the user has typed something and it doesn't
    // match (or is not on track to match) the expected literal CONFIRMATION_TEXT.
    // A correct-so-far prefix (e.g. "del", "Del") is not an error — we just leave
    // the submit button disabled until the full word lands. Comparison is
    // case-insensitive so any casing of the word confirms.
    const matchesPrefixSoFar = expectedLower.startsWith(value.toLowerCase());
    setShowError(value.length > 0 && !matchesPrefixSoFar);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && isValid && !loading) {
      onConfirm();
    }
  };

  const handleConfirm = () => {
    if (isValid && !loading) {
      onConfirm();
    }
  };

  return (
    <AlertDialog.Root open={open} onOpenChange={loading ? undefined : onClose}>
      <AlertDialog.Content maxWidth="450px">
        {/* Header with Close Button */}
        <Flex align="start" justify="between" mb="2">
          <AlertDialog.Title size="5">{entityStrings ? entityStrings.TITLE : SharedStrings.DELETE_CONFIRMATION.DELETE(entityType)}</AlertDialog.Title>
          <IconButton
            variant="ghost"
            size="1"
            color="gray"
            aria-label={CommonStrings.ACTIONS.CLOSE}
            disabled={loading}
            onClick={onClose}
          >
            <X size={16} />
          </IconButton>
        </Flex>

        {/* Warning Message — surface the entity name here so the user sees what they're
            deleting now that the acknowledgement is a fixed literal word. */}
        <AlertDialog.Description size="2" mb="4">
          {entityName ? (
            SharedStrings.DELETE_CONFIRMATION.ACTION_CANNOT_UNDONE_NAMED(<Text as="span" weight="bold">{entityName}</Text>)
          ) : (
            <>{entityStrings ? entityStrings.MESSAGE : SharedStrings.DELETE_CONFIRMATION.ACTION_CANNOT_UNDONE_TYPED(entityType)}</>
          )}
        </AlertDialog.Description>

        {/* Optional caller-provided context (resource details, related counts, etc.) */}
        {children && <Box mb="4">{children}</Box>}

        {/* Highlighted Warning Box */}
        <Box
          p="3"
          mb="4"
          style={{
            backgroundColor: 'var(--red-2)',
            border: '1px solid var(--red-6)',
            borderRadius: '6px',
          }}
        >
          <Text size="2" weight="medium" style={{ display: 'block' }}>
            {CONFIRMATION_TEXT}
          </Text>
          <Text size="1" color="gray">
            {SharedStrings.DELETE_CONFIRMATION.TYPE_CONFIRM_DELETION}
          </Text>
        </Box>

        {/* Verification Input */}
        <Box mb="4">
          <Flex align="center" gap="1" mb="2">
            <Text size="2" weight="bold">
              {SharedStrings.DELETE_CONFIRMATION.ACKNOWLEDGEMENT}
            </Text>
            <Text size="2" style={{ color: 'var(--red-11)' }}>
              *
            </Text>
          </Flex>
          <TextField.Root
            ref={inputRef}
            size="2"
            value={confirmationText}
            onChange={handleChange}
            onKeyDown={handleKeyDown}
            placeholder={SharedStrings.DELETE_CONFIRMATION.TYPE_CONFIRM(CONFIRMATION_TEXT)}
            disabled={loading}
            color={showError ? 'red' : undefined}
            style={
              showError
                ? {
                    outline: '2px solid var(--red-9)',
                    outlineOffset: '-2px',
                  }
                : undefined
            }
            aria-label={SharedStrings.DELETE_CONFIRMATION.ACKNOWLEDGEMENT_ARIA}
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
              {SharedStrings.DELETE_CONFIRMATION.CONFIRMATION_TEXT_PROVIDED_INCORRECT}
            </Text>
          )}
        </Box>

        {/* Action Buttons */}
        <Flex gap="3" mt="4" justify="end">
          <AlertDialog.Cancel>
            <Button variant="surface" size="2" disabled={loading}>
              {CommonStrings.ACTIONS.CANCEL}
            </Button>
          </AlertDialog.Cancel>
          <AlertDialog.Action>
            <Button
              variant="solid"
              color="red"
              size="2"
              onClick={handleConfirm}
              disabled={!isValid || loading}
            >
              {loading ? SharedStrings.DELETE_CONFIRMATION.DELETING : CommonStrings.ACTIONS.DELETE}
            </Button>
          </AlertDialog.Action>
        </Flex>
      </AlertDialog.Content>
    </AlertDialog.Root>
  );
}

export default DeleteConfirmationModal;
