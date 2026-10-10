
import React from 'react';
import {AsButton, AsFontAwesomeIcon} from '@globalart/astrum-react-shared-components';
import {faExternalLinkAlt, faUserCircle} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import './SupportRequest.scss';

export default function SupportRequest() {
  function openSupportRequestPage() {
    window.open('https://links.globalart.dev/products/astrum/pro/support-request', '_blank');
  }

  return <Page>
    <PageHeader>
      <PageTitle
          icon={UIStrings.SUPPORT_REQUEST.MENU.icon}
          text={UIStrings.SUPPORT_REQUEST.MENU.text}
          description={UIStrings.SUPPORT_REQUEST.MENU.description}
      />
    </PageHeader>
    <ContentBody className='as-support-request'>
      <Section>
        <p>{UIStrings.SUPPORT_REQUEST.DESCRIPTION}</p>
        <p><a href="#admin/support/supportzip">{UIStrings.SUPPORT_REQUEST.ATTACH_SUPPORT_ZIP}</a></p>
        <AsButton variant="primary" onClick={() => openSupportRequestPage()}>
          <AsFontAwesomeIcon icon={faExternalLinkAlt}/>
          <span>{UIStrings.SUPPORT_REQUEST.ACTIONS.submitRequest}</span>
        </AsButton>
      </Section>
    </ContentBody>
  </Page>;
}
