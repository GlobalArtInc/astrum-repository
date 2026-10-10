
import React, { useCallback, useEffect } from 'react';
import {useMachine} from '@xstate/react';

import {
  AsTile,
  AsLoadWrapper,
  AsFooter,
  AsButtonBar,
  AsButton,
  AsH2,
  AsTooltip,
} from '@globalart/react-shared-components';

import {faIdCardAlt} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  Page
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

import SslCertificatesDetails from './SslCertificatesDetails';
import Machine from './SslCertificatesDetailsFormMachine';

import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';
import {canDeleteCertificate} from './SslCertificatesHelper';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';

const ADMIN = ROUTE_NAMES.ADMIN;
const {
  SSL_CERTIFICATES: {FORM: LABELS},
} = UIStrings;

export default function SslCertificatesDetailsForm() {
  const router = useRouter();
  const {state: routerState, params} = useCurrentStateAndParams();
  const onDone = useCallback(() => router.stateService.go(ADMIN.SECURITY.SSLCERTIFICATES.LIST));
  const itemId = params?.itemId;
  const canDelete = canDeleteCertificate();

  const [state, send] = useMachine(Machine, {
    context: {
      pristineData: {
        id: decodeURIComponent(itemId),
      },
    },
    actions: {
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone,
    },
    devTools: true,
  });

  useEffect(() => {
    // we should not render edit form if itemId is not provided
    if (routerState.name === ADMIN.SECURITY.SSLCERTIFICATES.EDIT && !itemId) {
      router.stateService.go(ROUTE_NAMES.MISSING_ROUTE);
    }
  }, [itemId]);

  const {data = {}, loadError} = state.context;
  const isLoading = state.matches('loading');

  const retry = () => send({type: 'RETRY'});

  const confirmDelete = () => {
    if (canDelete) {
      send({type: 'CONFIRM_DELETE'});
    }
  };

  return (
    <Page className="as-ssl-certificate">
      <PageHeader>
        <PageTitle
          icon={faIdCardAlt}
          text={LABELS.DETAILS_TITLE(data.subjectCommonName || '')}
          description={LABELS.DETAILS_DESCRIPTION}
        />
      </PageHeader>
      <ContentBody className="as-ssl-certificate-form">
        <AsTile>
          <AsTile.Content>
            <AsLoadWrapper
              loading={isLoading}
              error={loadError}
              retryHandler={retry}
            >
              <AsH2>{LABELS.SECTIONS.CERTIFICATE}</AsH2>
              <SslCertificatesDetails data={data} />
              <AsFooter>
                <AsButtonBar>
                  <AsButton type="button" onClick={onDone}>
                    {UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}
                  </AsButton>
                  <AsTooltip title={!canDelete && UIStrings.PERMISSION_ERROR}>
                    <AsButton
                        type="button"
                        variant="primary"
                        onClick={confirmDelete}
                        className={!canDelete ? 'disabled' : ''}
                    >
                      {LABELS.BUTTONS.DELETE}
                    </AsButton>
                  </AsTooltip>
                </AsButtonBar>
              </AsFooter>
            </AsLoadWrapper>
          </AsTile.Content>
        </AsTile>
      </ContentBody>
    </Page>
  );
}
