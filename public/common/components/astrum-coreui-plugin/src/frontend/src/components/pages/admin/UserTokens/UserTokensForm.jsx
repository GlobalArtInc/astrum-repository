
import React from 'react';
import {useActor} from '@xstate/react';
import {faKey} from '@fortawesome/free-solid-svg-icons';

import {ExtJS, Permissions, FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsButton,
  AsCheckbox,
  AsFieldset,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsH2,
  AsP,
  AsStatefulForm,
  AsTextInput,
  AsTooltip
} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import UserTokensResetModal from './UserTokensResetModal';
import UserTokenExpiryChangesModal from './UserTokenExpiryChangesModal';

const {
  USER_TOKEN_CONFIGURATION: {
    CAPTION,
    HELP_TEXT,
    USER_TOKENS_CHECKBOX,
    REPOSITORY_AUTHENTICATION_CHECKBOX,
    EXPIRATION_CHECKBOX,
    USER_TOKEN_EXPIRY,
    RESET_ALL_TOKENS_BUTTON
  },
  SETTINGS: {DISCARD_BUTTON_LABEL}
} = UIStrings;

export default function UserTokensForm({service}) {
  const canDelete = ExtJS.checkPermission(Permissions.USER_TOKENS_USERS.DELETE);

  const [state, send] = useActor(service);

  const resetConfirmation = state.matches('resetConfirmation');

  const {
    isPristine,
    data: {enabled, expirationEnabled},
    pristineData: {enabled: enabledSaved}
  } = state.context;

  const discard = () => send({type: 'RESET'});
  const showResetConfirmation = () => send({type: 'RESET_CONFIRMATION'});
  const setEnabled = (value) => send({type: 'SET_ENABLED', value});
  const showUserTokenExpiryChangesModal = state.matches('showUserTokenExpiryChangesModal');
  const save = () => send('SAVE');
  const closeModal = () => send('CLOSE');

  return (
    <>
      <AsStatefulForm
        {...FormUtils.formProps(state, send)}
        additionalFooterBtns={
          <>
            {canDelete && enabledSaved && (
              <AsButton type="button" variant="tertiary" onClick={showResetConfirmation}>
                <AsFontAwesomeIcon icon={faKey} />
                <span>{RESET_ALL_TOKENS_BUTTON}</span>
              </AsButton>
            )}
            <AsTooltip title={FormUtils.discardTooltip({isPristine})}>
              <AsButton
                type="button"
                variant="secondary"
                className={isPristine ? 'disabled' : ''}
                onClick={discard}
              >
                {DISCARD_BUTTON_LABEL}
              </AsButton>
            </AsTooltip>
          </>
        }
      >
        <AsH2>{CAPTION}</AsH2>
        <AsP>{HELP_TEXT}</AsP>

        <AsFieldset label={USER_TOKENS_CHECKBOX.LABEL}>
          <AsCheckbox {...FormUtils.checkboxProps('enabled', state)} onChange={setEnabled}>
            {USER_TOKENS_CHECKBOX.DESCRIPTION}
          </AsCheckbox>
        </AsFieldset>

        <AsFieldset label={REPOSITORY_AUTHENTICATION_CHECKBOX.LABEL}>
          <AsCheckbox
            {...FormUtils.checkboxProps('protectContent', state)}
            onChange={FormUtils.handleUpdate('protectContent', send)}
            disabled={!enabled}
          >
            {REPOSITORY_AUTHENTICATION_CHECKBOX.DESCRIPTION}
          </AsCheckbox>
        </AsFieldset>

        <AsFieldset label={EXPIRATION_CHECKBOX.LABEL}>
          <AsCheckbox
              {...FormUtils.checkboxProps('expirationEnabled', state)}
              onChange={FormUtils.handleUpdate('expirationEnabled', send)}
              disabled={!enabled}
          >
            {EXPIRATION_CHECKBOX.DESCRIPTION}
          </AsCheckbox>
        </AsFieldset>

        <AsFormGroup
          label={USER_TOKEN_EXPIRY.LABEL}
          sublabel={USER_TOKEN_EXPIRY.SUBLABEL}
          isRequired={expirationEnabled}
        >
          <AsTextInput
            {...FormUtils.fieldProps('expirationDays', state)}
            className="as-text-input--short as-user-tokens-expiry-field"
            onChange={FormUtils.handleUpdate('expirationDays', send)}
            disabled={!enabled || !expirationEnabled}
          />
        </AsFormGroup>
      </AsStatefulForm>

      {resetConfirmation && <UserTokensResetModal service={service} />}
      {showUserTokenExpiryChangesModal && <UserTokenExpiryChangesModal onClose={closeModal}
                                                                       onConfirm={save}
                                                                       expirationEnabled={expirationEnabled}/>}
    </>
  );
}
