
import React from 'react';
import {useMachine} from '@xstate/react';
import {
  NxReadOnly,
  NxLoadWrapper,
  NxAccordion,
  NxTile,
  NxInfoAlert,
  NxH2,
} from '@globalart/react-shared-components';

import {ReadOnlyField} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {CONFIGURATION: LABELS} = UIStrings.HTTP;

import Machine from './HttpMachine';

export default function HttpReadOnly() {
  const [current, send] = useMachine(Machine, {devTools: true});
  const {data, loadError} = current.context;
  const {httpEnabled, httpAuthEnabled, httpsEnabled, httpsAuthEnabled} = data;
  const isLoading = current.matches('loading');
  const nonProxyHosts = data.nonProxyHosts || [];

  const retry = () => send({type: 'RETRY'});

  return (
    <NxLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
      <NxH2>{LABELS.READ_ONLY.LABEL}</NxH2>
      <NxInfoAlert>{LABELS.READ_ONLY.WARNING}</NxInfoAlert>
      <NxReadOnly>
        <ReadOnlyField
          label={LABELS.USER_AGENT.LABEL}
          value={data.userAgentSuffix}
        />
        <ReadOnlyField label={LABELS.TIMEOUT.LABEL} value={data.timeout} />
        <ReadOnlyField label={LABELS.ATTEMPTS.LABEL} value={data.retries} />
        {httpEnabled && (
          <>
            <ReadOnlyField
              label={LABELS.PROXY.HTTP_HOST}
              value={data.httpHost}
            />
            <ReadOnlyField
              label={LABELS.PROXY.HTTP_PORT}
              value={data.httpPort}
            />
          </>
        )}
        {httpEnabled && httpAuthEnabled && (
          <NxTile.Content className="nx-tile-content--accordion-container">
            <NxAccordion open={data.httpAuthEnabled}>
              <NxAccordion.Header>
                <NxAccordion.Title>
                  {LABELS.PROXY.HTTP_AUTHENTICATION}
                </NxAccordion.Title>
              </NxAccordion.Header>
              <ReadOnlyField
                label={LABELS.PROXY.USERNAME}
                value={data.httpAuthUsername}
              />
              <ReadOnlyField
                label={LABELS.PROXY.HOST_NAME}
                value={data.httpAuthNtlmHost}
              />
              <ReadOnlyField
                label={LABELS.PROXY.DOMAIN}
                value={data.httpAuthNtlmDomain}
              />
            </NxAccordion>
          </NxTile.Content>
        )}
        {httpsEnabled && (
          <>
            <ReadOnlyField
              label={LABELS.PROXY.HTTPS_HOST}
              value={data.httpsHost}
            />
            <ReadOnlyField
              label={LABELS.PROXY.HTTPS_PORT}
              value={data.httpsPort}
            />
          </>
        )}
        {httpsEnabled && httpsAuthEnabled && (
          <NxTile.Content className="nx-tile-content--accordion-container">
            <NxAccordion open={data.httpsAuthEnabled}>
              <NxAccordion.Header>
                <NxAccordion.Title>
                  {LABELS.PROXY.HTTPS_AUTHENTICATION}
                </NxAccordion.Title>
              </NxAccordion.Header>
              <ReadOnlyField
                label={LABELS.PROXY.USERNAME}
                value={data.httpsAuthUsername}
              />
              <ReadOnlyField
                label={LABELS.PROXY.HOST_NAME}
                value={data.httpsAuthNtlmHost}
              />
              <ReadOnlyField
                label={LABELS.PROXY.DOMAIN}
                value={data.httpsAuthNtlmDomain}
              />
            </NxAccordion>
          </NxTile.Content>
        )}
        {nonProxyHosts.length > 0 && (
          <>
            <NxReadOnly.Label>{LABELS.EXCLUDE.LABEL}</NxReadOnly.Label>
            {nonProxyHosts.map((item) => (
              <NxReadOnly.Data key={item}>{item}</NxReadOnly.Data>
            ))}
          </>
        )}
      </NxReadOnly>
    </NxLoadWrapper>
  );
}
