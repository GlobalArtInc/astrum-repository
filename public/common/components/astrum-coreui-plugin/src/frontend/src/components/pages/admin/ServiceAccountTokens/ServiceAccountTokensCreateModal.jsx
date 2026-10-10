
import React, {useState, useEffect} from 'react';
import {
  AsButton,
  AsButtonBar,
  AsErrorAlert,
  AsFooter,
  AsFormGroup,
  AsFormSelect,
  AsH2,
  AsModal,
  AsSubmitMask,
  AsTextInput,
  AsWarningAlert,
} from '@globalart/astrum-react-shared-components';
import UIStrings from '../../../../constants/UIStrings';
import {EXPIRATION_OPTIONS} from './ServiceAccountTokensHelper';

const LABELS = UIStrings.SERVICE_ACCOUNT_TOKENS.CREATE_MODAL;

const NAME_PATTERN = /^[a-zA-Z0-9_-]+$/;

function validateName(name, existingNames) {
  const trimmed = name.trim();
  if (name.length > 0 && !NAME_PATTERN.test(trimmed)) return LABELS.NAME_INVALID_CHARS_ERROR;
  if (trimmed.length > 0 && existingNames.includes(trimmed)) {
    return LABELS.NAME_DUPLICATE_ERROR(trimmed);
  }
  return null;
}

export default function ServiceAccountTokensCreateModal({onClose, onCreate, roles, rolesError = null, existingNames = [], isCreating = false}) {
  const [name, setName] = useState('');
  const [roleId, setRoleId] = useState('');
  const [expirationDays, setExpirationDays] = useState(String(EXPIRATION_OPTIONS[0].value));
  const [description, setDescription] = useState('');

  const parsedExpiration = parseInt(expirationDays, 10);
  const nameError = validateName(name, existingNames);
  const isValid = name.trim().length > 0 && roleId.length > 0 && !nameError && !rolesError;
  const isNeverExpires = parsedExpiration === -1;

  useEffect(() => {
    if (isNeverExpires) {
      const alertEl = document.getElementById('as-sa-never-expires-warning');
      if (alertEl) {
        alertEl.scrollIntoView({behavior: 'smooth', block: 'nearest'});
        // Move focus so screen readers announce the warning. tabIndex=-1 keeps
        // it out of the tab order while allowing programmatic focus.
        alertEl.focus({preventScroll: true});
      }
    }
  }, [isNeverExpires]);

  function handleSubmit() {
    if (!isValid || isCreating) return;
    const payload = {
      name: name.trim(),
      roleId,
      description: description.trim(),
    };
    if (!isNeverExpires) {
      payload.expirationDays = parsedExpiration;
    }
    onCreate(payload);
  }

  const submitDisabled = !isValid || isCreating;

  return (
    <AsModal onCancel={isCreating ? undefined : onClose} aria-labelledby="create-sa-token-modal">
      <AsModal.Header>
        <AsH2 id="create-sa-token-modal">{LABELS.TITLE}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsFormGroup label={LABELS.NAME_LABEL} sublabel={LABELS.NAME_DESCRIPTION} isRequired>
          <AsTextInput
            value={name}
            onChange={(v) => setName(v.slice(0, 128))}
            validatable={true}
            isPristine={name.length === 0}
            validationErrors={nameError ? [nameError] : null}
            inputAttributes={{maxLength: 128}}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.ROLE_LABEL} sublabel={LABELS.ROLE_DESCRIPTION} isRequired>
          <AsFormSelect value={roleId} onChange={setRoleId} disabled={!!rolesError}>
            <option value="">{LABELS.ROLE_PLACEHOLDER}</option>
            {roles.map((role) => (
              <option key={role.id} value={role.id}>{role.name}</option>
            ))}
          </AsFormSelect>
        </AsFormGroup>
        {rolesError && (
          <AsErrorAlert>{rolesError}</AsErrorAlert>
        )}
        <AsFormGroup label={LABELS.EXPIRATION_LABEL} sublabel={LABELS.EXPIRATION_DESCRIPTION} isRequired>
          <AsFormSelect value={expirationDays} onChange={setExpirationDays}>
            {EXPIRATION_OPTIONS.map((opt) => (
              <option key={opt.value} value={String(opt.value)}>{opt.label}</option>
            ))}
          </AsFormSelect>
        </AsFormGroup>
        <div className="as-sa-description-wrapper">
          <AsFormGroup label={LABELS.DESCRIPTION_LABEL} sublabel={LABELS.DESCRIPTION_DESCRIPTION}>
            <AsTextInput
              value={description}
              onChange={(v) => setDescription(v.slice(0, 256))}
              type="textarea"
              inputAttributes={{maxLength: 256}}
            />
          </AsFormGroup>
          <span className="as-sa-char-counter">{description.length} / 256</span>
        </div>
        {isNeverExpires && (
          <AsWarningAlert
            id="as-sa-never-expires-warning"
            tabIndex={-1}
            className="as-sa-sr-focus-target"
          >
            {LABELS.NEVER_EXPIRES_WARNING}
          </AsWarningAlert>
        )}
      </AsModal.Content>
      <AsFooter>
        <AsButtonBar>
          <AsButton type="button" onClick={onClose} disabled={isCreating}>
            {LABELS.CANCEL_BUTTON}
          </AsButton>
          <AsButton
            type="button"
            variant="primary"
            disabled={submitDisabled}
            onClick={handleSubmit}
          >
            {LABELS.CREATE_BUTTON}
          </AsButton>
        </AsButtonBar>
      </AsFooter>
      {isCreating && <AsSubmitMask message={LABELS.CREATING_MASK} />}
    </AsModal>
  );
}
