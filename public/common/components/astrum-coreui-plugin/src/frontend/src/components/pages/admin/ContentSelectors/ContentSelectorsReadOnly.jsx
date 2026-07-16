
import React from 'react';
import {useActor} from '@xstate/react';

import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxReadOnly,
  NxTile,
  NxFooter,
  NxButtonBar,
  NxInfoAlert
} from '@globalart/react-shared-components';

import {Section} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

export default function ContentSelectorsReadOnly({service, onDone}) {
  const [state] = useActor(service);

  const {data} = state.context;

  const cancel = () => onDone();

  return <Section className="as-content-selectors-read-only">
    <NxTile.Content>
      <NxInfoAlert>{UIStrings.SETTINGS.READ_ONLY.WARNING}</NxInfoAlert>
      <NxReadOnly>
        <NxReadOnly.Label>{UIStrings.CONTENT_SELECTORS.NAME_LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{FormUtils.fieldProps('name', state).value}</NxReadOnly.Data>
        <NxReadOnly.Label>{UIStrings.CONTENT_SELECTORS.TYPE_LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{data.type?.toUpperCase()}</NxReadOnly.Data>
        <NxReadOnly.Label>{UIStrings.CONTENT_SELECTORS.DESCRIPTION_LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{FormUtils.fieldProps('description', state).value}</NxReadOnly.Data>
        <NxReadOnly.Label>{UIStrings.CONTENT_SELECTORS.EXPRESSION_LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{FormUtils.fieldProps('expression', state).value}</NxReadOnly.Data>
      </NxReadOnly>
    </NxTile.Content>
    <NxFooter>
      <NxButtonBar>
        <NxButton type="button" onClick={cancel}>{UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}</NxButton>
      </NxButtonBar>
    </NxFooter>
  </Section>
}
