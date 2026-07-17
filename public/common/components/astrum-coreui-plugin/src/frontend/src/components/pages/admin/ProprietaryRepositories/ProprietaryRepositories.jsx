
import React from 'react';
import {ExtJS} from '@globalart/astrum-ui-plugin';

import {faDoorClosed} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import ProprietaryRepositoriesForm from './ProprietaryRepositoriesForm';
import ProprietaryRepositoriesReadOnly from './ProprietaryRepositoriesReadOnly';

import UIStrings from '../../../../constants/UIStrings';

export default function ProprietaryRepositories() {
  const canEdit = ExtJS.checkPermission('astrum:settings:update');

  return <Page>
    <PageHeader>
      <PageTitle
          icon={UIStrings.PROPRIETARY_REPOSITORIES.MENU.icon}
          text={UIStrings.PROPRIETARY_REPOSITORIES.MENU.text}
          description={UIStrings.PROPRIETARY_REPOSITORIES.MENU.description}
      />
    </PageHeader>
    <ContentBody className="as-proprietary_repositories">
      <Section>
        {canEdit ? <ProprietaryRepositoriesForm/> : <ProprietaryRepositoriesReadOnly/>}
      </Section>
    </ContentBody>
  </Page>;
}
