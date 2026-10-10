
import React from 'react';

import {AsH2, AsModal, AsButton, AsFooter, AsButtonBar} from '@globalart/astrum-react-shared-components';
import {useRouter} from '@uirouter/react';
import {ExtJS} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import {ROUTE_NAMES} from '../../../../routerConfig/routeNames/routeNames';

const ADMIN = ROUTE_NAMES.ADMIN;

const {
  SSL_CERTIFICATES: {
    ADD_FORM: {
      MODAL: {HEADER, CONTENT, VIEW_BUTTON}
    }
  },
  SETTINGS: {CANCEL_BUTTON_LABEL}
} = UIStrings;

export default function SslCertificatesAlreadyExistsModal({certificateId, cancel}) {
  const router = useRouter();
  const handleCancel = () => {
    ExtJS.setDirtyStatus('SslCertificatesAddFormMachine', false);
    cancel();
  };
  const goToCertificate = () => {
    ExtJS.setDirtyStatus('SslCertificatesAddFormMachine', false);
    router.stateService.go(ADMIN.SECURITY.SSLCERTIFICATES.EDIT, {itemId: encodeURIComponent(certificateId)});
  };

  return (
    <AsModal onCancel={cancel} variant="narrow">
      <AsModal.Header>
        <AsH2>{HEADER}</AsH2>
      </AsModal.Header>
      <AsModal.Content>{CONTENT}</AsModal.Content>
      <AsFooter>
        <AsButtonBar>
          <AsButton onClick={handleCancel}>{CANCEL_BUTTON_LABEL}</AsButton>
          <AsButton onClick={goToCertificate} variant="primary">
            {VIEW_BUTTON}
          </AsButton>
        </AsButtonBar>
      </AsFooter>
    </AsModal>
  );
}
