
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsFormGroup,
  AsCheckbox,
  AsTextInput,
  AsAccordion,
  AsStatefulAccordion,
  AsFieldset
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function GenericHttpReqConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {userAgentSuffix, retries, timeout, enableCircularRedirects, enableCookies} =
    currentParent.context.data.httpClient.connection || {};

  const isOpen = Boolean(userAgentSuffix || retries || timeout || enableCircularRedirects || enableCookies);
  
  return (
    <AsStatefulAccordion
      className="as-form-accordion-http-request"
      defaultOpen={isOpen}
    >
      <AsAccordion.Header>
        <AsAccordion.Title>{EDITOR.REQUEST_SETTINGS_CAPTION}</AsAccordion.Title>
      </AsAccordion.Header>

      <AsFormGroup
        label={EDITOR.USER_AGENT_LABEL}
        sublabel={EDITOR.USER_AGEN_SUBLABEL}
        className="as-form-group-user-agent"
      >
        <AsTextInput
          {...FormUtils.fieldProps('httpClient.connection.userAgentSuffix', currentParent)}
          onChange={FormUtils.handleUpdate('httpClient.connection.userAgentSuffix', sendParent)}
        />
      </AsFormGroup>

      <AsFormGroup
        label={EDITOR.RETRIES_LABEL}
        sublabel={EDITOR.RETRIES_SUBLABEL}
        className="as-form-group-retries"
      >
        <AsTextInput
          {...FormUtils.fieldProps('httpClient.connection.retries', currentParent)}
          onChange={FormUtils.handleUpdate('httpClient.connection.retries', sendParent)}
        />
      </AsFormGroup>

      <AsFormGroup
        label={EDITOR.TIMEOUT_LABEL}
        sublabel={EDITOR.TIMEOUT_SUBLABEL}
        className="as-form-group-timeout"
      >
        <AsTextInput
          {...FormUtils.fieldProps('httpClient.connection.timeout', currentParent)}
          onChange={FormUtils.handleUpdate('httpClient.connection.timeout', sendParent)}
        />
      </AsFormGroup>

      <AsFieldset label={EDITOR.REDIRECTS_LABEL} className="as-form-group-redirects">
        <AsCheckbox
          {...FormUtils.checkboxProps(
            'httpClient.connection.enableCircularRedirects',
            currentParent
          )}
          onChange={FormUtils.handleUpdate(
            'httpClient.connection.enableCircularRedirects',
            sendParent
          )}
        >
          {EDITOR.ENABLED_CHECKBOX_DESCR}
        </AsCheckbox>
      </AsFieldset>

      <AsFieldset label={EDITOR.COOKIES_LABEL} className="as-form-group-cookies">
        <AsCheckbox
          {...FormUtils.checkboxProps('httpClient.connection.enableCookies', currentParent)}
          onChange={FormUtils.handleUpdate('httpClient.connection.enableCookies', sendParent)}
        >
          {EDITOR.ENABLED_CHECKBOX_DESCR}
        </AsCheckbox>
      </AsFieldset>
    </AsStatefulAccordion>
  );
}
