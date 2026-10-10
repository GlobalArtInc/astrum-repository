

import React from 'react';
import {useMachine} from '@xstate/react';
import {
  AsStatefulAccordion,
  AsAccordion,
  AsButton,
  AsCheckbox,
  AsFieldset,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsFormRow,
  AsH2,
  AsList,
  AsStatefulForm,
  AsTextInput,
  AsTile,
  AsTooltip
} from '@globalart/react-shared-components';
import {ExtJS, FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';
import {faTrashAlt, faPlusCircle} from '@fortawesome/free-solid-svg-icons';
import UIStrings from '../../../../constants/UIStrings';

const {CONFIGURATION: LABELS} = UIStrings.HTTP;

import HttpMachine from './HttpMachine';

export default function HttpForm() {
  const [current, send] = useMachine(HttpMachine, {devTools: true});
  const {
    data: {
      nonProxyHosts,
      httpsEnabled,
      httpEnabled,
      httpAuthEnabled,
      httpsAuthEnabled,
      httpHost,
      httpsHost
    },
    isPristine,
    nonProxyHost = ''
  } = current.context;

  const list = nonProxyHosts || [];

  const isProxyEnabled = httpEnabled || httpsEnabled;
  const isHttpAuthEnabled =  httpEnabled && httpAuthEnabled;
  const isHttpsAuthEnabled =  httpsEnabled && httpsAuthEnabled;

  const discard = () => send({type: 'RESET'});

  const removeNonProxyHost = (index) =>
    send({type: 'REMOVE_NON_PROXY_HOST', index});

  const addNonProxyHost = () => send({type: 'ADD_NON_PROXY_HOST'});

  const toggleAuthentication = (name, value) =>
    send({type: 'TOGGLE_AUTHENTICATION', name, value});

  const handleHttpCheckbox = () => send({type: 'TOGGLE_HTTP_PROXY'});

  const handleHttpsCheckbox = () => send({type: 'TOGGLE_HTTPS_PROXY'});

  const handleEnter = (event) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      addNonProxyHost();
    }
  };

  const handleSetNonProxyHost = (value) => send({type: 'SET_NON_PROXY_HOST', value});

  const validateNonProxyHost = () =>
    ValidationUtils.validateWhiteSpace(nonProxyHost) || 
      (nonProxyHosts.includes(nonProxyHost) ? LABELS.EXCLUDE.ALREADY_ADDED : null)

  const timeoutPlaceholder = ExtJS.state().getValue('requestTimeout');
  const retryCountPlaceholder = ExtJS.state().getValue('retryCount');

  return (
    <AsStatefulForm
      {...FormUtils.formProps(current, send)}
      additionalFooterBtns={
        <AsTooltip title={FormUtils.discardTooltip({isPristine})}>
          <AsButton
            type="button"
            className={isPristine && 'disabled'}
            onClick={discard}
          >
            {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
          </AsButton>
        </AsTooltip>
      }
    >
      <AsFormGroup
        label={LABELS.USER_AGENT.LABEL}
        sublabel={LABELS.USER_AGENT.SUB_LABEL}
      >
        <AsTextInput
          className="as-text-input--long"
          {...FormUtils.fieldProps('userAgentSuffix', current)}
          onChange={FormUtils.handleUpdate('userAgentSuffix', send)}
        />
      </AsFormGroup>
      <AsFormGroup
        label={LABELS.TIMEOUT.LABEL}
        sublabel={LABELS.TIMEOUT.SUB_LABEL}
      >
        <AsTextInput
          className="as-text-input--short"
          {...FormUtils.fieldProps('timeout', current)}
          onChange={FormUtils.handleUpdate('timeout', send)}
          placeholder={timeoutPlaceholder.value}
        />
      </AsFormGroup>
      <AsFormGroup
        label={LABELS.ATTEMPTS.LABEL}
        sublabel={LABELS.ATTEMPTS.SUB_LABEL}
      >
        <AsTextInput
          className="as-text-input--short"
          {...FormUtils.fieldProps('retries', current)}
          onChange={FormUtils.handleUpdate('retries', send)}
          placeholder={retryCountPlaceholder}
        />
      </AsFormGroup>
      <AsH2>{LABELS.PROXY.LABEL}</AsH2>
      <AsFieldset label="">
        <AsCheckbox
          {...FormUtils.checkboxProps('httpEnabled', current)}
          onChange={handleHttpCheckbox}
        >
          {LABELS.PROXY.HTTP_CHECKBOX}
        </AsCheckbox>
      </AsFieldset>
      <AsFormGroup
        label={LABELS.PROXY.HTTP_HOST}
        sublabel={LABELS.PROXY.SUB_LABEL}
        isRequired={httpEnabled}
      >
        <AsTextInput
          className="as-text-input--long"
          {...FormUtils.fieldProps('httpHost', current)}
          onChange={FormUtils.handleUpdate('httpHost', send)}
          disabled={!httpEnabled}
        />
      </AsFormGroup>
      <AsFormGroup 
        label={LABELS.PROXY.HTTP_PORT} 
        isRequired={!!httpHost}
      >
        <AsTextInput
          className="as-text-input--short"
          {...FormUtils.fieldProps('httpPort', current)}
          onChange={FormUtils.handleUpdate('httpPort', send)}
          disabled={!httpEnabled || !httpHost}
        />
      </AsFormGroup>
      <AsTile.Content className="as-tile-content--accordion-container">
        <AsStatefulAccordion 
          title="http-authentication"
          defaultOpen={httpAuthEnabled}
        >
          <AsAccordion.Header>
            <AsAccordion.Title>
              {LABELS.PROXY.HTTP_AUTHENTICATION}
            </AsAccordion.Title>
          </AsAccordion.Header>
          <AsFieldset label="">
            <AsCheckbox
              {...FormUtils.checkboxProps('httpAuthEnabled', current)}
              onChange={() => toggleAuthentication('httpAuthEnabled')}
              disabled={!httpEnabled}
            >
              {LABELS.PROXY.HTTP_AUTH_CHECKBOX}
            </AsCheckbox>
          </AsFieldset>
          <AsFormRow>
            <>
              <AsFormGroup 
                label={LABELS.PROXY.USERNAME} 
                isRequired={httpAuthEnabled}
              >
                <AsTextInput
                  className="as-text-input"
                  {...FormUtils.fieldProps('httpAuthUsername', current)}
                  onChange={FormUtils.handleUpdate('httpAuthUsername', send)}
                  disabled={!isHttpAuthEnabled}
                />
              </AsFormGroup>
              <AsFormGroup label={LABELS.PROXY.PASSWORD}>
                <AsTextInput
                  className="as-text-input"
                  {...FormUtils.fieldProps('httpAuthPassword', current)}
                  onChange={FormUtils.handleUpdate('httpAuthPassword', send)}
                  type="password"
                  disabled={!isHttpAuthEnabled}
                />
              </AsFormGroup>
            </>
          </AsFormRow>
          <AsFormGroup label={LABELS.PROXY.HOST_NAME}>
            <AsTextInput
              className="as-text-input--long"
              {...FormUtils.fieldProps('httpAuthNtlmHost', current)}
              onChange={FormUtils.handleUpdate('httpAuthNtlmHost', send)}
              disabled={!isHttpAuthEnabled}
            />
          </AsFormGroup>
          <AsFormGroup label={LABELS.PROXY.DOMAIN}>
            <AsTextInput
              className="as-text-input--long"
              {...FormUtils.fieldProps('httpAuthNtlmDomain', current)}
              onChange={FormUtils.handleUpdate('httpAuthNtlmDomain', send)}
              disabled={!isHttpAuthEnabled}
            />
          </AsFormGroup>
        </AsStatefulAccordion>
      </AsTile.Content>
      <AsFieldset label="">
        <AsCheckbox
          {...FormUtils.checkboxProps('httpsEnabled', current)}
          onChange={handleHttpsCheckbox}
        >
          {LABELS.PROXY.HTTPS_CHECKBOX}
        </AsCheckbox>
      </AsFieldset>
      <AsFormGroup
        label={LABELS.PROXY.HTTPS_HOST}
        sublabel={LABELS.PROXY.SUB_LABEL}
        isRequired={httpsEnabled}
      >
        <AsTextInput
          className="as-text-input--long"
          {...FormUtils.fieldProps('httpsHost', current)}
          onChange={FormUtils.handleUpdate('httpsHost', send)}
          disabled={!httpsEnabled}
        />
      </AsFormGroup>
      <AsFormGroup 
        label={LABELS.PROXY.HTTPS_PORT} 
        isRequired={!!httpsHost}
      >
        <AsTextInput
          className="as-text-input--short"
          {...FormUtils.fieldProps('httpsPort', current)}
          onChange={FormUtils.handleUpdate('httpsPort', send)}
          disabled={!httpsEnabled || !httpsHost}
        />
      </AsFormGroup>
      <AsTile.Content className="as-tile-content--accordion-container">
        <AsStatefulAccordion
          title="https-authentication"
          defaultOpen={httpsAuthEnabled}
        >
          <AsAccordion.Header>
            <AsAccordion.Title>
              {LABELS.PROXY.HTTPS_AUTHENTICATION}
            </AsAccordion.Title>
          </AsAccordion.Header>
          <AsFieldset label="">
            <AsCheckbox
              {...FormUtils.checkboxProps('httpsAuthEnabled', current)}
              onChange={() => toggleAuthentication('httpsAuthEnabled')}
              disabled={!httpsEnabled}
            >
              {LABELS.PROXY.HTTPS_AUTH_CHECKBOX}
            </AsCheckbox>
          </AsFieldset>
          <AsFormRow>
            <>
              <AsFormGroup 
                label={LABELS.PROXY.USERNAME} 
                isRequired={httpsAuthEnabled}
              >
                <AsTextInput
                  className="as-text-input"
                  {...FormUtils.fieldProps('httpsAuthUsername', current)}
                  onChange={FormUtils.handleUpdate('httpsAuthUsername', send)}
                  disabled={!isHttpsAuthEnabled}
                />
              </AsFormGroup>
              <AsFormGroup label={LABELS.PROXY.PASSWORD}>
                <AsTextInput
                  className="as-text-input"
                  {...FormUtils.fieldProps('httpsAuthPassword', current)}
                  onChange={FormUtils.handleUpdate('httpsAuthPassword', send)}
                  type="password"
                  disabled={!isHttpsAuthEnabled}
                />
              </AsFormGroup>
            </>
          </AsFormRow>
          <AsFormGroup label={LABELS.PROXY.HOST_NAME}>
            <AsTextInput
              className="as-text-input--long"
              {...FormUtils.fieldProps('httpsAuthNtlmHost', current)}
              onChange={FormUtils.handleUpdate('httpsAuthNtlmHost', send)}
              disabled={!isHttpsAuthEnabled}
            />
          </AsFormGroup>
          <AsFormGroup label={LABELS.PROXY.DOMAIN}>
            <AsTextInput
              className="as-text-input--long"
              {...FormUtils.fieldProps('httpsAuthNtlmDomain', current)}
              onChange={FormUtils.handleUpdate('httpsAuthNtlmDomain', send)}
              disabled={!isHttpsAuthEnabled}
            />
          </AsFormGroup>
        </AsStatefulAccordion>
      </AsTile.Content>
      {isProxyEnabled && (
        <AsFormRow>
          <>
            <AsFormGroup
              label={LABELS.EXCLUDE.LABEL}
              sublabel={LABELS.EXCLUDE.SUB_LABEL}
            >
              <AsTextInput
                className="as-text-input--long"
                id="nonProxyHost"
                name="nonProxyHost"
                value={nonProxyHost}
                isPristine={nonProxyHost === ''}
                onChange={handleSetNonProxyHost}
                onKeyDown={handleEnter}
                validatable
                validationErrors={validateNonProxyHost()}
              />
            </AsFormGroup>
            <AsButton
              variant="icon-only"
              title={LABELS.EXCLUDE.ADD}
              onClick={addNonProxyHost}
              type="button"
            >
              <AsFontAwesomeIcon icon={faPlusCircle} />
            </AsButton>
          </>
        </AsFormRow>
      )}
      {isProxyEnabled && list.length > 0 && (
        <AsList>
          {list.map((item, index) => (
            <AsList.Item key={item}>
              <AsList.Text>{item}</AsList.Text>
              <AsList.Actions>
                <AsButton
                  title={LABELS.EXCLUDE.REMOVE}
                  variant="icon-only"
                  onClick={() => removeNonProxyHost(index)}
                >
                  <AsFontAwesomeIcon icon={faTrashAlt} />
                </AsButton>
              </AsList.Actions>
            </AsList.Item>
          ))}
        </AsList>
      )}
    </AsStatefulForm>
  );
}
