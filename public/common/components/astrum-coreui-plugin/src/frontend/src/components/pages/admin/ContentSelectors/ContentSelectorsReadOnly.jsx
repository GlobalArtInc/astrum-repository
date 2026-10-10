
import React from 'react';
import {useActor} from '@xstate/react';

import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsReadOnly,
  AsTile,
  AsFooter,
  AsButtonBar,
  AsInfoAlert
} from '@globalart/react-shared-components';

import {Section} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

export default function ContentSelectorsReadOnly({service, onDone}) {
  const [state] = useActor(service);

  const {data} = state.context;

  const cancel = () => onDone();

  return <Section className="as-content-selectors-read-only">
    <AsTile.Content>
      <AsInfoAlert>{UIStrings.SETTINGS.READ_ONLY.WARNING}</AsInfoAlert>
      <AsReadOnly>
        <AsReadOnly.Label>{UIStrings.CONTENT_SELECTORS.NAME_LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{FormUtils.fieldProps('name', state).value}</AsReadOnly.Data>
        <AsReadOnly.Label>{UIStrings.CONTENT_SELECTORS.TYPE_LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{data.type?.toUpperCase()}</AsReadOnly.Data>
        <AsReadOnly.Label>{UIStrings.CONTENT_SELECTORS.DESCRIPTION_LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{FormUtils.fieldProps('description', state).value}</AsReadOnly.Data>
        <AsReadOnly.Label>{UIStrings.CONTENT_SELECTORS.EXPRESSION_LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{FormUtils.fieldProps('expression', state).value}</AsReadOnly.Data>
      </AsReadOnly>
    </AsTile.Content>
    <AsFooter>
      <AsButtonBar>
        <AsButton type="button" onClick={cancel}>{UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}</AsButton>
      </AsButtonBar>
    </AsFooter>
  </Section>
}
