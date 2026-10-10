
import React, { useCallback } from 'react';
import {useMachine} from '@xstate/react';

import {
  AsTile,
  AsH2,
  AsFormGroup,
  AsTextInput,
  AsFieldset,
  AsRadio,
  AsStatefulForm,
} from '@globalart/react-shared-components';
import {FormUtils} from '@globalart/astrum-ui-plugin';

import {faIdCardAlt} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  Page
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

import SslCertificatesAlreadyExistsModal from './SslCertificatesAlreadyExistsModal';
import SslCertificatesDetails from "./SslCertificatesDetails";

import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';
import { useRouter } from '@uirouter/react';

import Machine, {SOURCES} from "./SslCertificatesAddFormMachine";

const ADMIN = ROUTE_NAMES.ADMIN;
const {ADD_FORM: {CAPTION, LOAD_BUTTON, PEM, SERVER}, MENU, FORM} = UIStrings.SSL_CERTIFICATES;

export default function SslCertificatesAddForm() {
  const router = useRouter();
  const onDone = useCallback(() => router.stateService.go(ADMIN.SECURITY.SSLCERTIFICATES.LIST));
  const [state, send] = useMachine(Machine, {
    actions: {
      onSaveSuccess: onDone,
    },
    devTools: true,
  });

  const {source, data} = state.context;
  const {id, inTrustStore} = data;

  const showAddForm = FormUtils.isInState(state, ['loaded', 'loadingDetails']);
  const showDetails = FormUtils.isInState(state, ['previewDetails', 'saving']);
  const submitMaskState = FormUtils.submitMaskState(state, ['loadingDetails', 'saving']);
  const submitMaskMessage = showAddForm ? UIStrings.LOADING : UIStrings.SAVING;

  const updateSource = (value) => send({type: 'SET_SOURCE', value});
  const onSubmit = () => showAddForm ? send({type: 'LOAD_DETAILS'}) : send({type: 'ADD_CERTIFICATE'});

  return (
    <Page className="as-ssl-certificate">
      <PageHeader>
        <PageTitle icon={faIdCardAlt} {...MENU} />
      </PageHeader>
      <ContentBody className="as-ssl-certificate-add-form">
        <AsTile>
          <AsTile.Content>
            <AsStatefulForm
                {...FormUtils.formProps(state, send)}
                onSubmit={onSubmit}
                submitBtnText={showAddForm ? LOAD_BUTTON : FORM.BUTTONS.ADD}
                onCancel={onDone}
                submitMaskMessage={submitMaskMessage}
                submitMaskState={submitMaskState}
            >
              <AsH2>{CAPTION}</AsH2>
              {showAddForm && (
                <>
                  <AsFieldset label="">
                    <AsRadio
                        name="source"
                        value={SOURCES.REMOTE_HOST}
                        onChange={updateSource}
                        isChecked={source === SOURCES.REMOTE_HOST}
                    >
                      {SERVER.RADIO_DESCRIPTION}
                    </AsRadio>
                  </AsFieldset>
                  <AsFormGroup label={SERVER.LABEL} isRequired={source === SOURCES.REMOTE_HOST}>
                    <AsTextInput
                        {...FormUtils.fieldProps('remoteHostUrl', state)}
                        onChange={FormUtils.handleUpdate('remoteHostUrl', send)}
                        className="as-text-input--long"
                        disabled={source === SOURCES.PEM}
                    />
                  </AsFormGroup>
                  <AsFieldset label="">
                    <AsRadio
                        name="source"
                        value={SOURCES.PEM}
                        onChange={updateSource}
                        isChecked={source === SOURCES.PEM}
                    >
                      {PEM.RADIO_DESCRIPTION}
                    </AsRadio>
                  </AsFieldset>
                  <AsFormGroup label={PEM.LABEL} isRequired ={source === SOURCES.PEM}>
                    <AsTextInput
                        type="textarea"
                        {...FormUtils.fieldProps('pemContent', state)}
                        onChange={FormUtils.handleUpdate('pemContent', send)}
                        className="as-text-input--long"
                        placeholder={PEM.PLACEHOLDER}
                        disabled={source === SOURCES.REMOTE_HOST}
                    />
                  </AsFormGroup>
                </>
              )}
              {showDetails && <SslCertificatesDetails data={data} />}
            </AsStatefulForm>

          </AsTile.Content>
        </AsTile>
      </ContentBody>

      {inTrustStore && <SslCertificatesAlreadyExistsModal certificateId={id} cancel={onDone} />}
    </Page>
  );
}
