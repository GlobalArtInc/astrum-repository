
import React from 'react';

import {AsFieldset, AsRadio, AsFormGroup, AsTextInput} from '@globalart/astrum-react-shared-components';
import {FormUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../../constants/UIStrings';

import {DOCKER_INDEX_TYPES} from '../RepositoryFormConfig';

const {INDEX} = UIStrings.REPOSITORIES.EDITOR.DOCKER;

export const DOCKER_HUB_URL = 'https://index.docker.io/';

export default function DockerIndexConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {indexType} = currentParent.context.data.dockerProxy;

  const updateIndexType = (value) => {
    sendParent({type: 'UPDATE', name: 'dockerProxy.indexType', value});
    const indexUrl = value === DOCKER_INDEX_TYPES.registry ? null : DOCKER_HUB_URL;
    sendParent({type: 'UPDATE', name: 'dockerProxy.indexUrl', value: indexUrl});
  };

  return (
    <>
      <AsFieldset label={INDEX.LABEL} isRequired>
        <AsRadio
          name="indexType"
          value={DOCKER_INDEX_TYPES.registry}
          onChange={updateIndexType}
          isChecked={indexType === DOCKER_INDEX_TYPES.registry}
        >
          {INDEX.OPTIONS.REGISTRY}
        </AsRadio>
        <AsRadio
          name="indexType"
          value={DOCKER_INDEX_TYPES.hub}
          onChange={updateIndexType}
          isChecked={indexType === DOCKER_INDEX_TYPES.hub}
        >
          {INDEX.OPTIONS.HUB}
        </AsRadio>
        <AsRadio
          name="indexType"
          value={DOCKER_INDEX_TYPES.custom}
          onChange={updateIndexType}
          isChecked={indexType === DOCKER_INDEX_TYPES.custom}
        >
          {INDEX.OPTIONS.CUSTOM}
        </AsRadio>
      </AsFieldset>

      {indexType !== DOCKER_INDEX_TYPES.registry && (
        <AsFormGroup
          label={INDEX.URL.LABEL}
          className="as-form-group-docker-index-url"
          isRequired
        >
          <AsTextInput
            {...FormUtils.fieldProps('dockerProxy.indexUrl', currentParent)}
            onChange={FormUtils.handleUpdate('dockerProxy.indexUrl', sendParent)}
            disabled={indexType === DOCKER_INDEX_TYPES.hub}
            placeholder={INDEX.URL.PLACEHOLDER}
          />
        </AsFormGroup>
      )}
    </>
  );
}
