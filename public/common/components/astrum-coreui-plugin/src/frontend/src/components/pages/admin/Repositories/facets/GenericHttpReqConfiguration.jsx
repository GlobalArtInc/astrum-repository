
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {
  NxFormGroup,
  NxCheckbox,
  NxTextInput,
  NxAccordion,
  NxStatefulAccordion,
  NxFieldset
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function GenericHttpReqConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {userAgentSuffix, retries, timeout, enableCircularRedirects, enableCookies} =
    currentParent.context.data.httpClient.connection || {};

  const isOpen = Boolean(userAgentSuffix || retries || timeout || enableCircularRedirects || enableCookies);
  
  return (
    <NxStatefulAccordion
      className="as-form-accordion-http-request"
      defaultOpen={isOpen}
    >
      <NxAccordion.Header>
        <NxAccordion.Title>{EDITOR.REQUEST_SETTINGS_CAPTION}</NxAccordion.Title>
      </NxAccordion.Header>

      <NxFormGroup
        label={EDITOR.USER_AGENT_LABEL}
        sublabel={EDITOR.USER_AGEN_SUBLABEL}
        className="as-form-group-user-agent"
      >
        <NxTextInput
          {...FormUtils.fieldProps('httpClient.connection.userAgentSuffix', currentParent)}
          onChange={FormUtils.handleUpdate('httpClient.connection.userAgentSuffix', sendParent)}
        />
      </NxFormGroup>

      <NxFormGroup
        label={EDITOR.RETRIES_LABEL}
        sublabel={EDITOR.RETRIES_SUBLABEL}
        className="as-form-group-retries"
      >
        <NxTextInput
          {...FormUtils.fieldProps('httpClient.connection.retries', currentParent)}
          onChange={FormUtils.handleUpdate('httpClient.connection.retries', sendParent)}
        />
      </NxFormGroup>

      <NxFormGroup
        label={EDITOR.TIMEOUT_LABEL}
        sublabel={EDITOR.TIMEOUT_SUBLABEL}
        className="as-form-group-timeout"
      >
        <NxTextInput
          {...FormUtils.fieldProps('httpClient.connection.timeout', currentParent)}
          onChange={FormUtils.handleUpdate('httpClient.connection.timeout', sendParent)}
        />
      </NxFormGroup>

      <NxFieldset label={EDITOR.REDIRECTS_LABEL} className="as-form-group-redirects">
        <NxCheckbox
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
        </NxCheckbox>
      </NxFieldset>

      <NxFieldset label={EDITOR.COOKIES_LABEL} className="as-form-group-cookies">
        <NxCheckbox
          {...FormUtils.checkboxProps('httpClient.connection.enableCookies', currentParent)}
          onChange={FormUtils.handleUpdate('httpClient.connection.enableCookies', sendParent)}
        >
          {EDITOR.ENABLED_CHECKBOX_DESCR}
        </NxCheckbox>
      </NxFieldset>
    </NxStatefulAccordion>
  );
}
