
import React from 'react';
import {useMachine} from '@xstate/react';
import UIStrings from '../../../../constants/UIStrings';
import {ExtJS} from '@globalart/astrum-ui-plugin';
import {NxLoadWrapper} from '@globalart/react-shared-components';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import SupportZipForm from './SupportZipForm';
import SupportZipResponse from './SupportZipResponse';
import SupportZipResponseHA from './SupportZipResponseHA';
import SupportZipMachine from './SupportZipMachine';
import {faArchive} from '@fortawesome/free-solid-svg-icons';

export default function SupportZipSingleNode() {
  const [current, send] = useMachine(SupportZipMachine, {devTools: true});
  const isLoading = current.matches('creatingSupportZips') || current.matches('creatingHaSupportZips');
  const isCreated = current.matches('supportZipsCreated');
  const isHaZipsCreated = current.matches('haSupportZipsCreated');
  const {params, response, createError} = current.context;
  const isHa = ExtJS.state().isClustered();

  function retry() {
    if (isHa) {
      send({type: 'RETRY_HA'});
    }
    else {
      send({type: 'RETRY'});
    }
  }

  function setParams(id, value) {
    send({
      type: 'UPDATE',
      params: {
        ...params,
        [id]: value
      }
    });
  }

  function submit(event) {
    event.preventDefault();
    send({type: 'CREATE_SUPPORT_ZIPS'});
  }

  function hazips(event) {
    event.preventDefault();
    send({type: 'CREATE_HA_SUPPORT_ZIPS'});
  }

  function download(event, filename) {
    event.preventDefault();
    const url = ExtJS.urlOf(`service/rest/wonderland/download/${filename}`);
    ExtJS.downloadUrl(url);
  }

  return <Page>
    <PageHeader>
      {/* icon does not match what's in constants faArchive vs faFileArchive */}
      <PageTitle
          icon={faArchive}
          text={UIStrings.SUPPORT_ZIP.MENU.text}
          description={UIStrings.SUPPORT_ZIP.MENU.description}
      />
    </PageHeader>
    <ContentBody className="as-support-zip">
      <Section>
        {!(isCreated || isHaZipsCreated) &&
        <NxLoadWrapper loading={isLoading} error={createError ? `${createError}` : null} retryHandler={retry}>
          <SupportZipForm params={params} setParams={setParams} submit={submit} clustered={isHa} hazips={hazips}/>
        </NxLoadWrapper>
        }
        {isCreated && <SupportZipResponse response={response} download={download}/>}
        {isHaZipsCreated && <SupportZipResponseHA response={response} download={download}/>}
      </Section>
    </ContentBody>
  </Page>;
}
