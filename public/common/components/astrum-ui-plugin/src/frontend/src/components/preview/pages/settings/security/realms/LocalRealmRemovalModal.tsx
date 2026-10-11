

import React, { useState, useRef, useEffect } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { Box, Flex, Text, Heading } from '@radix-ui/themes';
import { AlertTriangle, X, ExternalLink } from 'lucide-react';
import { SettingsButton } from '../../../../shared/form';

import './LocalRealmRemovalModal.scss';
import RealmsStrings from '../../../../constants/pages/settings/security/RealmsStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const ACKNOWLEDGEMENT_STRING = RealmsStrings.LOCAL_REALM_REMOVAL.I_ACKNOWLEDGE;

export interface LocalRealmRemovalModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

/**
 * Modal dialog shown when user attempts to remove local authenticating or authorizing realms.
 * Requires user to type "I acknowledge" to confirm the action.
 */
export function LocalRealmRemovalModal({ open, onClose, onConfirm }: LocalRealmRemovalModalProps) {
  const [acknowledgement, setAcknowledgement] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus input when modal opens
  useEffect(() => {
    if (open && inputRef.current) {
      // Small delay to ensure DOM is ready
      setTimeout(() => inputRef.current?.focus(), 50);
    }
    // Reset acknowledgement when modal opens/closes
    if (!open) {
      setAcknowledgement('');
    }
  }, [open]);

  const isValid = acknowledgement === ACKNOWLEDGEMENT_STRING;

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && isValid) {
      onConfirm();
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setAcknowledgement(e.target.value);
  };

  return (
    <Dialog.Root open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="local-realm-removal-modal__overlay" />
        <Dialog.Content
          className="local-realm-removal-modal"
          aria-describedby="modal-description"
          data-testid="local-realm-removal-modal"
        >
          <Dialog.Title asChild>
            <Flex align="center" gap="2" className="local-realm-removal-modal__header">
              <AlertTriangle size={20} className="local-realm-removal-modal__icon" />
              <Heading as="h2" size="4" weight="medium">
                {RealmsStrings.LOCAL_REALM_REMOVAL.CONFIRM_REMOVAL_LOCAL_REALMS}
              </Heading>
            </Flex>
          </Dialog.Title>

          <Dialog.Close asChild>
            <button
              className="local-realm-removal-modal__close"
              aria-label={CommonStrings.ACTIONS.CLOSE}
              type="button"
            >
              <X size={16} />
            </button>
          </Dialog.Close>

          <Box className="local-realm-removal-modal__content">
            <Box className="local-realm-removal-modal__warning" id="modal-description">
              <AlertTriangle size={16} />
              <Text size="2">
                {RealmsStrings.LOCAL_REALM_REMOVAL.WARNING(
                  <a
                    href="http://links.globalart.dev/products/astrum/docs/realms"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="local-realm-removal-modal__link"
                  >
                    {RealmsStrings.LOCAL_REALM_REMOVAL.HELP_LINK}
                    <ExternalLink size={12} />
                  </a>
                )}
              </Text>
            </Box>

            <Box className="local-realm-removal-modal__form-group">
              <label htmlFor="acknowledgement-input" className="local-realm-removal-modal__label">
                <Text size="2" weight="medium">{RealmsStrings.LOCAL_REALM_REMOVAL.ACKNOWLEDGEMENT}</Text>
                <Text size="1" className="local-realm-removal-modal__required">*</Text>
              </label>
              <Text size="1" className="local-realm-removal-modal__sublabel">
                {RealmsStrings.LOCAL_REALM_REMOVAL.TYPE_ORDER_PROCEED_ACTION(ACKNOWLEDGEMENT_STRING)}
              </Text>
              <input
                ref={inputRef}
                id="acknowledgement-input"
                type="text"
                value={acknowledgement}
                onChange={handleChange}
                onKeyDown={handleKeyDown}
                placeholder={RealmsStrings.LOCAL_REALM_REMOVAL.PROVIDE_ACKNOWLEDGEMENT_PLACEHOLDER}
                className={`local-realm-removal-modal__input ${acknowledgement && !isValid ? 'local-realm-removal-modal__input--invalid' : ''}`}
                data-testid="acknowledgement-input"
              />
              {acknowledgement && !isValid && (
                <Text size="1" className="local-realm-removal-modal__error">
                  {RealmsStrings.LOCAL_REALM_REMOVAL.INVALID_ACKNOWLEDGEMENT}
                </Text>
              )}
            </Box>
          </Box>

          <Flex gap="2" justify="end" className="local-realm-removal-modal__footer">
            <SettingsButton variant="secondary" onClick={onClose} data-testid="cancel-button">
              {CommonStrings.ACTIONS.CANCEL}
            </SettingsButton>
            <SettingsButton
              variant="danger"
              onClick={onConfirm}
              disabled={!isValid}
              data-testid="confirm-button"
            >
              {CommonStrings.ACTIONS.CONFIRM}
            </SettingsButton>
          </Flex>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export default LocalRealmRemovalModal;
