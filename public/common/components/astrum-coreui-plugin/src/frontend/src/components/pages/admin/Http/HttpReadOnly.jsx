
import React from 'react';
import {useMachine} from '@xstate/react';
import {
  AsReadOnly,
  AsLoadWrapper,
  AsAccordion,
  AsTile,
  AsInfoAlert,
  AsH2,
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
    <AsLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
      <AsH2>{LABELS.READ_ONLY.LABEL}</AsH2>
      <AsInfoAlert>{LABELS.READ_ONLY.WARNING}</AsInfoAlert>
      <AsReadOnly>
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
          <AsTile.Content className="as-tile-content--accordion-container">
            <AsAccordion open={data.httpAuthEnabled}>
              <AsAccordion.Header>
                <AsAccordion.Title>
                  {LABELS.PROXY.HTTP_AUTHENTICATION}
                </AsAccordion.Title>
              </AsAccordion.Header>
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
            </AsAccordion>
          </AsTile.Content>
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
          <AsTile.Content className="as-tile-content--accordion-container">
            <AsAccordion open={data.httpsAuthEnabled}>
              <AsAccordion.Header>
                <AsAccordion.Title>
                  {LABELS.PROXY.HTTPS_AUTHENTICATION}
                </AsAccordion.Title>
              </AsAccordion.Header>
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
            </AsAccordion>
          </AsTile.Content>
        )}
        {nonProxyHosts.length > 0 && (
          <>
            <AsReadOnly.Label>{LABELS.EXCLUDE.LABEL}</AsReadOnly.Label>
            {nonProxyHosts.map((item) => (
              <AsReadOnly.Data key={item}>{item}</AsReadOnly.Data>
            ))}
          </>
        )}
      </AsReadOnly>
    </AsLoadWrapper>
  );
}
