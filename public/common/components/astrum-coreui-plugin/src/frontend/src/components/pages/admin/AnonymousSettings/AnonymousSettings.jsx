
import React from 'react';
import {
  ExtJS
} from '@globalart/astrum-ui-plugin';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import AnonymousSettingsForm from './AnonymousSettingsForm';
import AnonymousSettingsReadOnly from './AnonymousSettingsReadOnly';

import UIStrings from '../../../../constants/UIStrings';

export default function AnonymousSettings() {
  const canEdit = ExtJS.checkPermission('astrum:settings:update');

  if (!ExtJS.useUser()) {
    return null;
  }

  return <Page>
    <PageHeader>
      <PageTitle
          icon={UIStrings.ANONYMOUS_SETTINGS.MENU.icon}
          text={UIStrings.ANONYMOUS_SETTINGS.MENU.text}
          description={UIStrings.ANONYMOUS_SETTINGS.description }
      />
    </PageHeader>
    <ContentBody className='as-anonymous-settings'>
      <Section>
        {canEdit ? <AnonymousSettingsForm/> : <AnonymousSettingsReadOnly/>}
      </Section>
    </ContentBody>
  </Page>;
}
