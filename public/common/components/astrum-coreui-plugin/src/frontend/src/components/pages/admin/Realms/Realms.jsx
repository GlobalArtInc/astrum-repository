
import React from 'react';
import {ExtJS} from '@globalart/astrum-ui-plugin';

import {faDungeon} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import RealmsForm from './RealmsForm';
import RealmsReadOnly from './RealmsReadOnly';

import UIStrings from '../../../../constants/UIStrings';

export default function Realms() {
  const canEdit = ExtJS.checkPermission('astrum:settings:update');

  if (!ExtJS.useUser()) {
    return null;
  }

  return (
    <Page>
      <PageHeader>
        <PageTitle
            icon={UIStrings.REALMS.MENU.icon}
            text={UIStrings.REALMS.MENU.text}
            description={UIStrings.REALMS.MENU.description}
        />
      </PageHeader>
      <ContentBody className="as-realms">
        <Section>{canEdit ? <RealmsForm /> : <RealmsReadOnly />}</Section>
      </ContentBody>
    </Page>
  );
}
