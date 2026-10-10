
import React, {useCallback} from 'react';

import Axios from 'axios';

import {AsH2, AsCheckbox, AsFieldset, AsInfoAlert} from '@globalart/react-shared-components';
import {FormUtils, ExtJS} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../../constants/UIStrings';

import ToggleableTextInput from './ToggleableTextInput/ToggleableTextInput';

const {CONNECTORS} = UIStrings.REPOSITORIES.EDITOR.DOCKER;

export default function RepositoryConnectorsConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  const repositoryName = parentState.context.data.name;

  const isProEdition = ExtJS.isProEdition();

  const fetchSuggestedPort = useCallback(async (contextPropName) => {
    try {
      const docker = parentState.context.data?.docker;
      const otherPort = contextPropName === 'docker.httpPort' ? docker?.httpsPort : docker?.httpPort;
      const url = otherPort
        ? `service/rest/internal/ui/docker/suggest-port?exclude=${otherPort}`
        : 'service/rest/internal/ui/docker/suggest-port';
      const response = await Axios.get(url);
      sendParent({
        type: 'UPDATE',
        name: contextPropName,
        value: String(response.data),
      });
    }
    catch (e) {
      // Non-blocking: suggest-port failure should never prevent repo creation.
      // User can enter a port manually.
      console.warn('Could not suggest Docker connector port:', e?.response?.status ?? e?.message);
    }
  }, [parentState, sendParent]);

  return (
    <>
      <AsH2 className="as-docker-connectors-caption">{CONNECTORS.CAPTION}</AsH2>

      <p className="as-docker-connectors-help">{CONNECTORS.HELP}</p>

      {isProEdition && (
        <ToggleableTextInput
          parentMachine={parentMachine}
          contextPropName="docker.subdomain"
          label={CONNECTORS.SUBDOMAIN.LABEL}
          sublabel={CONNECTORS.SUBDOMAIN.SUBLABEL}
          defaultValue={repositoryName}
          placeholder={CONNECTORS.SUBDOMAIN.PLACEHOLDER}
          clearIfDisabled
          className="as-form-group-docker-connector-subdomain"
        />
      )}

      <ToggleableTextInput
        parentMachine={parentMachine}
        contextPropName="docker.httpPort"
        label={CONNECTORS.HTTP.LABEL}
        sublabel={CONNECTORS.HTTP.SUBLABEL}
        placeholder={CONNECTORS.HTTP.PLACEHOLDER}
        clearIfDisabled
        className="as-form-group-docker-connector-http-port"
        onToggle={(checked) => checked && fetchSuggestedPort('docker.httpPort')}
      />
      <ToggleableTextInput
        parentMachine={parentMachine}
        contextPropName="docker.httpsPort"
        label={CONNECTORS.HTTPS.LABEL}
        sublabel={CONNECTORS.HTTPS.SUBLABEL}
        placeholder={CONNECTORS.HTTPS.PLACEHOLDER}
        clearIfDisabled
        className="as-form-group-docker-connector-https-port"
        onToggle={(checked) => checked && fetchSuggestedPort('docker.httpsPort')}
      />

      <AsFieldset
        label={CONNECTORS.ALLOW_ANON_DOCKER_PULL.LABEL}
        className="as-form-group-force-basic-auth"
      >
        <AsCheckbox
          {...FormUtils.checkboxProps('docker.forceBasicAuth', parentState)}
          isChecked={!FormUtils.checkboxProps('docker.forceBasicAuth', parentState).isChecked}
          onChange={(checked) => FormUtils.handleUpdate('docker.forceBasicAuth', sendParent)(!checked)}
        >
          {CONNECTORS.ALLOW_ANON_DOCKER_PULL.DESCR}
        </AsCheckbox>
        <AsInfoAlert>{CONNECTORS.ALLOW_ANON_DOCKER_PULL.HELP}</AsInfoAlert>
      </AsFieldset>
    </>
  );
}
