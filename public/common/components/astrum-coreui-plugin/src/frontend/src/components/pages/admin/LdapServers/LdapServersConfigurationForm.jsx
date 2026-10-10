
import React, {forwardRef} from 'react';
import classNames from 'classnames';
import {
  FormUtils,
  ValidationUtils,
  UseAstrumTruststore,
} from '@globalart/astrum-ui-plugin';
import {useActor} from '@xstate/react';
import {
  AsButton,
  AsButtonBar,
  AsFormGroup,
  AsFormSelect,
  AsH2,
  AsP,
  AsStatefulForm,
  AsStatefulSubmitMask,
  AsTextInput,
  AsTile,
  AsInfoAlert,
  AsPageTitle,
  AsFontAwesomeIcon,
  AsTooltip,
  AsModal,
  AsFooter,
} from '@globalart/astrum-react-shared-components';
import {faTrashAlt} from '@fortawesome/free-solid-svg-icons';

import LdapServersModalPassword from './LdapServersModalPassword';

import UIStrings from '../../../../constants/UIStrings';

import {
  isSimpleAuth,
  isDigestAuth,
  isCramtAuth,
  isAnonymousAuth,
  validateUrlValues,
  generateUrl,
  canDelete,
} from './LdapServersHelper';

const {
  LDAP_SERVERS: {FORM: LABELS},
  SETTINGS,
  PERMISSION_ERROR,
} = UIStrings;

export default forwardRef(({actor, onDone}, ref) => {
  const [state, send] = useActor(actor);
  const {
    isEdit,
    data: {protocol, authScheme, host, port, name},
    isPristine,
    validationErrors,
  } = state.context;
  const isProtocolSelected = ValidationUtils.notBlank(protocol);
  const isAuthMethodSelected = ValidationUtils.notBlank(authScheme);
  const showRealmAuth = isDigestAuth(authScheme) || isCramtAuth(authScheme);
  const showAuth = showRealmAuth || isSimpleAuth(authScheme);
  const isInvalid = FormUtils.isInvalid(validationErrors);
  const canVerify = !isInvalid;
  const url = generateUrl('https', host, port);
  const urlMessage = generateUrl(protocol, host, port);
  const showUseTrustStore = validateUrlValues(protocol, host, port);
  const verifyingConnection = state.matches('verifyingConnection');
  const changingPassword = state.matches('changingPassword');
  const changingPasswordMessage = state.matches(
    'changingPassword.verifyingConnection'
  );
  const askingPassword =
    state.matches('askingPassword') ||
    (changingPassword && !changingPasswordMessage);
  const confirmingDelete = state.matches('confirmingDelete');

  const next = () => send({type: 'NEXT'});

  const verify = () => {
    if (canVerify) {
      send({type: 'VERIFY_CONNECTION'});
    }
  };

  const onDelete = () => {
    if (canDelete()) {
      send({type: 'DELETE_CONNECTION'});
    }
  };

  const updateProtocol = (value) =>
    send({type: 'UPDATE_PROTOCOL', value});

  const onChangePassword = () => send({type: 'CHANGE_PASSWORD'});

  const validations = () => {
    if (isEdit && isPristine) {
      return undefined;
    }

    return (
      validationErrors?.active || FormUtils.saveTooltip({isPristine, isInvalid})
    );
  };

  return (
    <AsTile.Content>
      <AsStatefulForm
        {...FormUtils.formProps(state, send)}
        validationErrors={validations()}
        onSubmit={next}
        submitBtnText={LABELS.NEXT}
        onCancel={onDone}
        additionalFooterBtns={
          <AsButtonBar>
            <AsButton
              type="button"
              onClick={verify}
              className={classNames({disabled: !canVerify})}
            >
              {LABELS.VERIFY_CONNECTION}
            </AsButton>
          </AsButtonBar>
        }
        ref={ref}
      >
        <AsPageTitle>
          <AsH2>{LABELS.CONFIGURATION}</AsH2>
          {isEdit && (
            <AsButtonBar>
              <AsTooltip title={!canDelete() && PERMISSION_ERROR}>
                <AsButton
                  onClick={onDelete}
                  className={classNames({disabled: !canDelete()})}
                  variant="tertiary"
                  type="button"
                >
                  <AsFontAwesomeIcon icon={faTrashAlt} />
                  <span>{LABELS.DELETE_BUTTON}</span>
                </AsButton>
              </AsTooltip>
            </AsButtonBar>
          )}
        </AsPageTitle>
        <AsFormGroup label={LABELS.NAME} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('name', state)}
            onChange={FormUtils.handleUpdate('name', send)}
          />
        </AsFormGroup>
        <AsH2>{LABELS.SETTINGS.LABEL}</AsH2>
        <AsP>{LABELS.SETTINGS.SUB_LABEL}</AsP>
        <AsFormGroup label={LABELS.PROTOCOL.LABEL} isRequired>
          <AsFormSelect
            {...FormUtils.fieldProps('protocol', state)}
            onChange={updateProtocol}
            className="as-form-select--short"
            validatable
          >
            <option value="" disabled={isProtocolSelected} />
            {Object.values(LABELS.PROTOCOL.OPTIONS).map((label) => (
              <option key={label} value={label}>
                {label}
              </option>
            ))}
          </AsFormSelect>
        </AsFormGroup>
        <AsFormGroup label={LABELS.HOSTNAME} isRequired>
          <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('host', state)}
            onChange={FormUtils.handleUpdate('host', send)}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.PORT} isRequired>
          <AsTextInput
            className="as-text-input--short"
            {...FormUtils.fieldProps('port', state)}
            onChange={FormUtils.handleUpdate('port', send)}
          />
        </AsFormGroup>
        {showUseTrustStore && (
          <UseAstrumTruststore
            remoteUrl={url}
            {...FormUtils.checkboxProps('useTrustStore', state)}
            onChange={FormUtils.handleUpdate('useTrustStore', send)}
          />
        )}
        <AsFormGroup
          label={LABELS.SEARCH.LABEL}
          sublabel={LABELS.SEARCH.SUB_LABEL}
          isRequired
        >
          <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('searchBase', state)}
            onChange={FormUtils.handleUpdate('searchBase', send)}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.AUTHENTICATION.LABEL} isRequired>
          <AsFormSelect
            {...FormUtils.fieldProps('authScheme', state)}
            onChange={FormUtils.handleUpdate('authScheme', send)}
          >
            <option value="" disabled={isAuthMethodSelected} />
            {Object.values(LABELS.AUTHENTICATION.OPTIONS).map((option) => (
              <option key={option.id} value={option.id}>
                {option.label}
              </option>
            ))}
          </AsFormSelect>
        </AsFormGroup>

        {showAuth && (
          <>
            {showRealmAuth && (
              <AsFormGroup
                label={LABELS.SASL_REALM.LABEL}
                sublabel={LABELS.SASL_REALM.SUB_LABEL}
              >
                <AsTextInput
                  {...FormUtils.fieldProps('authRealm', state)}
                  onChange={FormUtils.handleUpdate('authRealm', send)}
                />
              </AsFormGroup>
            )}
            <AsFormGroup
              label={LABELS.USERNAME.LABEL}
              sublabel={LABELS.USERNAME.SUB_LABEL}
              isRequired
            >
              <AsTextInput
                className="as-text-input--long"
                {...FormUtils.fieldProps('authUsername', state)}
                onChange={FormUtils.handleUpdate('authUsername', send)}
              />
            </AsFormGroup>
            {!isEdit && (
              <AsFormGroup
                label={LABELS.PASSWORD.LABEL}
                sublabel={LABELS.PASSWORD.SUB_LABEL}
                isRequired
              >
                <AsTextInput
                  {...FormUtils.fieldProps('authPassword', state)}
                  onChange={FormUtils.handleUpdate('authPassword', send)}
                  type="password"
                />
              </AsFormGroup>
            )}
            {isEdit && !isAnonymousAuth(authScheme) && (
              <AsFormGroup label="">
                <AsButton
                  variant="tertiary"
                  className="change-password"
                  type="button"
                  onClick={onChangePassword}
                >
                  {LABELS.CHANGE_PASSWORD}
                </AsButton>
              </AsFormGroup>
            )}
          </>
        )}

        <AsH2>{LABELS.CONNECTION_RULES.LABEL}</AsH2>
        <AsP>{LABELS.CONNECTION_RULES.SUB_LABEL}</AsP>
        <AsFormGroup
          label={LABELS.WAIT_TIMEOUT.LABEL}
          sublabel={LABELS.WAIT_TIMEOUT.SUB_LABEL}
          isRequired
        >
          <AsTextInput
            className="as-text-input--short"
            {...FormUtils.fieldProps('connectionTimeout', state)}
            onChange={FormUtils.handleUpdate('connectionTimeout', send)}
          />
        </AsFormGroup>
        <AsFormGroup
          label={LABELS.RETRY_TIMEOUT.LABEL}
          sublabel={LABELS.RETRY_TIMEOUT.SUB_LABEL}
          isRequired
        >
          <AsTextInput
            className="as-text-input--short"
            {...FormUtils.fieldProps('connectionRetryDelay', state)}
            onChange={FormUtils.handleUpdate('connectionRetryDelay', send)}
          />
        </AsFormGroup>
        <AsFormGroup
          label={LABELS.MAX_RETRIES.LABEL}
          sublabel={LABELS.MAX_RETRIES.SUB_LABEL}
          isRequired
        >
          <AsTextInput
            className="as-text-input--short"
            {...FormUtils.fieldProps('maxIncidentsCount', state)}
            onChange={FormUtils.handleUpdate('maxIncidentsCount', send)}
          />
        </AsFormGroup>
        {isEdit && <AsInfoAlert>{LABELS.ALERT}</AsInfoAlert>}
        {verifyingConnection && (
          <AsStatefulSubmitMask
            message={LABELS.VERIFYING_MESSAGE(urlMessage)}
          />
        )}
        {changingPasswordMessage && (
          <AsStatefulSubmitMask
            message={LABELS.CHANGING_PASSWORD_MESSAGE(urlMessage)}
          />
        )}
      </AsStatefulForm>
      {askingPassword && (
        <LdapServersModalPassword
          onCancel={() => send({type: 'CANCEL'})}
          onSubmit={({value}) =>
            send({type: 'DONE', name: 'authPassword', value})
          }
        />
      )}
      {confirmingDelete && (
        <AsModal aria-labelledby="modal-header-confirm-delete" variant="narrow">
          <AsModal.Header>
            <AsH2 id="modal-header-confirm-delete">
              {LABELS.MODAL_DELETE.LABEL}
            </AsH2>
          </AsModal.Header>
          <AsModal.Content>
            <AsP>{LABELS.VERIFY_DELETE_MESSAGE(name)}</AsP>
          </AsModal.Content>
          <AsFooter>
            <AsButtonBar>
              <AsButton onClick={() => send({type: 'CANCEL'})}>
                {SETTINGS.CANCEL_BUTTON_LABEL}
              </AsButton>
              <AsButton onClick={() => send({type: 'ACCEPT'})} variant="primary">
                {LABELS.MODAL_DELETE.CONFIRM}
              </AsButton>
            </AsButtonBar>
          </AsFooter>
        </AsModal>
      )}
    </AsTile.Content>
  );
});
