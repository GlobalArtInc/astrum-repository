
import React from 'react';
import {
  ContentBody,
  ExtJS,
  Page,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';
import {NxTile} from '@globalart/react-shared-components';

import HttpForm from './HttpForm';
import HttpReadOnly from './HttpReadOnly';

import UIStrings from '../../../../constants/UIStrings';

import './Http.scss';

export default function Http() {
  const canEdit = ExtJS.checkPermission('astrum:settings:update');

  if (!ExtJS.useUser()) {
    return null;
  }

  return (
    <Page>
      <PageHeader>
        <PageTitle
            icon={UIStrings.HTTP.MENU.icon}
            text={UIStrings.HTTP.MENU.text}
            description={UIStrings.HTTP.MENU.description}
        />
      </PageHeader>
      <ContentBody className="as-http">
        <NxTile>{canEdit ? <HttpForm /> : <HttpReadOnly />}</NxTile>
      </ContentBody>
    </Page>
  );
}
