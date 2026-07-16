
import React from 'react';
import {useMachine} from '@xstate/react';
import {
  NxReadOnly,
  NxLoadWrapper,
  NxInfoAlert,
  NxH2,
} from '@globalart/react-shared-components';

import {FormUtils, ReadOnlyField} from '@globalart/astrum-ui-plugin';

import Machine from './EmailServerMachine';

import UIStrings from '../../../../constants/UIStrings';

const {FORM: LABELS, READ_ONLY} = UIStrings.EMAIL_SERVER;
const {readOnlyCheckboxValueLabel} = FormUtils;

export default function EmailServerReadOnly() {
  const [current, send] = useMachine(Machine, {devTools: true});
  const {data, loadError} = current.context;
  const isLoading = current.matches('loading');
  const enabled = readOnlyCheckboxValueLabel(data.enabled);
  const astrumTrustStoreEnabled = readOnlyCheckboxValueLabel(
    data.astrumTrustStoreEnabled
  );
  const retry = () => send({type: 'RETRY'});

  const renderData = (label, value) => {
    const enable = value ? 'ENABLE' : 'NOT_ENABLE';
    return <NxReadOnly.Data>{READ_ONLY[enable][label]}</NxReadOnly.Data>;
  };

  return (
    <NxLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
      <NxH2>{LABELS.SECTIONS.SETUP}</NxH2>
      <NxInfoAlert>{UIStrings.SETTINGS.READ_ONLY.WARNING}</NxInfoAlert>
      <NxReadOnly>
        <ReadOnlyField label={LABELS.ENABLED.LABEL} value={enabled} />
        {data.enabled && (
          <>
            <ReadOnlyField label={LABELS.HOST.LABEL} value={data.host} />
            <ReadOnlyField label={LABELS.PORT.LABEL} value={data.port} />
            <ReadOnlyField
              label={UIStrings.USE_TRUST_STORE.LABEL}
              value={astrumTrustStoreEnabled}
            />
            <ReadOnlyField
              label={LABELS.USERNAME.LABEL}
              value={data.username}
            />
            <ReadOnlyField
              label={LABELS.FROM_ADDRESS.LABEL}
              value={data.fromAddress}
            />
            <ReadOnlyField
              label={LABELS.SUBJECT_PREFIX.LABEL}
              value={data.subjectPrefix}
            />
            <NxReadOnly.Label>{LABELS.SSL_TLS_OPTIONS.LABEL}</NxReadOnly.Label>
            {renderData('ENABLE_STARTTLS', data.startTlsEnabled)}
            {renderData('REQUIRE_STARTTLS', data.startTlsRequired)}
            {renderData('ENABLE_SSL_TLS', data.sslOnConnectEnabled)}
            {renderData('IDENTITY_CHECK', data.sslServerIdentityCheckEnabled)}
          </>
        )}
      </NxReadOnly>
    </NxLoadWrapper>
  );
}
