
import React from 'react';
import {FormUtils, UseAstrumTruststore} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsCheckbox,
  AsFieldset,
  AsFormGroup,
  AsFormRow,
  AsH2,
  AsStatefulForm,
  AsTextInput,
  AsTile,
  AsTooltip,
} from '@globalart/astrum-react-shared-components';
import {isEmpty} from 'ramda';

import UIStrings from '../../../../constants/UIStrings';

const {EMAIL_SERVER: {FORM: LABELS}} = UIStrings;

export default function EmailServerForm({ parentMachine }) {
  const [state, send] = parentMachine;
  const {
    data,
    isPristine
  } = state.context;

  const discard = () => send({type: 'RESET'});

  const remoteUrl = !isEmpty(data.host) ? `https://${data.host}:${data.port}` : '';

  const updateHost = (value) => FormUtils.handleUpdate('host', send, 'UPDATE_AND_CLEAR_PASSWORD')(value);
  const updatePort = (value) => FormUtils.handleUpdate('port', send, 'UPDATE_AND_CLEAR_PASSWORD')(value);

  return <AsStatefulForm
      {...FormUtils.formProps(state, send)}
      additionalFooterBtns={<>
        <AsTooltip title={FormUtils.discardTooltip({isPristine})}>
          <AsButton type="button" className={isPristine && 'disabled'} onClick={discard}>
            {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
          </AsButton>
        </AsTooltip>
      </>}
  >
    <AsTile.Content>
      <AsH2>{LABELS.SECTIONS.SETUP}</AsH2>
      <AsFormGroup label={LABELS.ENABLED.LABEL}>
        <AsCheckbox
            {...FormUtils.checkboxProps('enabled', state)}
            onChange={FormUtils.handleUpdate('enabled', send)}
        >
          {LABELS.ENABLED.SUB_LABEL}
        </AsCheckbox>
      </AsFormGroup>
      <AsFormGroup label={LABELS.HOST.LABEL} isRequired>
        <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('host', state)}
            onChange={updateHost}
        />
      </AsFormGroup>
      <AsFormGroup label={LABELS.PORT.LABEL} isRequired>
        <AsTextInput
            className="as-text-input--short"
            {...FormUtils.fieldProps('port', state)}
            onChange={updatePort}
        />
      </AsFormGroup>
      <UseAstrumTruststore
        remoteUrl={remoteUrl}
        {...FormUtils.checkboxProps('astrumTrustStoreEnabled', state)}
        onChange={FormUtils.handleUpdate('astrumTrustStoreEnabled', send)}
      />
      <AsFormRow>
        <AsFormGroup label={LABELS.USERNAME.LABEL}>
          <AsTextInput
              {...FormUtils.fieldProps('username', state)}
              onChange={FormUtils.handleUpdate('username', send)}
          />
        </AsFormGroup>
        <AsFormGroup label={LABELS.PASSWORD.LABEL}>
          <AsTextInput
              type="password"
              autoComplete="new-password"
              {...FormUtils.fieldProps('password', state)}
              onChange={FormUtils.handleUpdate('password', send)}
          />
        </AsFormGroup>
      </AsFormRow>
      <AsFormGroup label={LABELS.FROM_ADDRESS.LABEL} isRequired>
        <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('fromAddress', state)}
            onChange={FormUtils.handleUpdate('fromAddress', send)}
        />
      </AsFormGroup>
      <AsFormGroup label={LABELS.SUBJECT_PREFIX.LABEL}>
        <AsTextInput
            className="as-text-input--long"
            {...FormUtils.fieldProps('subjectPrefix', state)}
            onChange={FormUtils.handleUpdate('subjectPrefix', send)}
        />
      </AsFormGroup>
      <AsFieldset label={LABELS.SSL_TLS_OPTIONS.LABEL}>
        <AsCheckbox
            {...FormUtils.checkboxProps('startTlsEnabled', state)}
            onChange={FormUtils.handleUpdate('startTlsEnabled', send)}
        >
          {LABELS.SSL_TLS_OPTIONS.OPTIONS.ENABLE_STARTTLS}
        </AsCheckbox>
        <AsCheckbox
            {...FormUtils.checkboxProps('startTlsRequired', state)}
            onChange={FormUtils.handleUpdate('startTlsRequired', send)}
        >
          {LABELS.SSL_TLS_OPTIONS.OPTIONS.REQUIRE_STARTTLS}
        </AsCheckbox>
        <AsCheckbox
            {...FormUtils.checkboxProps('sslOnConnectEnabled', state)}
            onChange={FormUtils.handleUpdate('sslOnConnectEnabled', send)}
        >
          {LABELS.SSL_TLS_OPTIONS.OPTIONS.ENABLE_SSL_TLS}
        </AsCheckbox>
        <AsCheckbox
            {...FormUtils.checkboxProps('sslCheckServerIdentityEnabled', state)}
            onChange={FormUtils.handleUpdate('sslCheckServerIdentityEnabled', send)}
        >
          {LABELS.SSL_TLS_OPTIONS.OPTIONS.IDENTITY_CHECK}
        </AsCheckbox>
      </AsFieldset>
    </AsTile.Content>
  </AsStatefulForm>;
}
