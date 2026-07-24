
import React from 'react';

import {NxFieldset, NxRadio, NxFormGroup, NxTextInput} from '@globalart/react-shared-components';
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
      <NxFieldset label={INDEX.LABEL} isRequired>
        <NxRadio
          name="indexType"
          value={DOCKER_INDEX_TYPES.registry}
          onChange={updateIndexType}
          isChecked={indexType === DOCKER_INDEX_TYPES.registry}
        >
          {INDEX.OPTIONS.REGISTRY}
        </NxRadio>
        <NxRadio
          name="indexType"
          value={DOCKER_INDEX_TYPES.hub}
          onChange={updateIndexType}
          isChecked={indexType === DOCKER_INDEX_TYPES.hub}
        >
          {INDEX.OPTIONS.HUB}
        </NxRadio>
        <NxRadio
          name="indexType"
          value={DOCKER_INDEX_TYPES.custom}
          onChange={updateIndexType}
          isChecked={indexType === DOCKER_INDEX_TYPES.custom}
        >
          {INDEX.OPTIONS.CUSTOM}
        </NxRadio>
      </NxFieldset>

      {indexType !== DOCKER_INDEX_TYPES.registry && (
        <NxFormGroup
          label={INDEX.URL.LABEL}
          className="as-form-group-docker-index-url"
          isRequired
        >
          <NxTextInput
            {...FormUtils.fieldProps('dockerProxy.indexUrl', currentParent)}
            onChange={FormUtils.handleUpdate('dockerProxy.indexUrl', sendParent)}
            disabled={indexType === DOCKER_INDEX_TYPES.hub}
            placeholder={INDEX.URL.PLACEHOLDER}
          />
        </NxFormGroup>
      )}
    </>
  );
}
