
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxTextInput, NxFormSelect, NxReadOnly} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {NUGET} = UIStrings.REPOSITORIES.EDITOR;

const NUGET_VERSIONS = {
  V2: 'NuGet V2',
  V3: 'NuGet V3'
};

export default function NugetProxyConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {
    pristineData: {name},
    data: {url}
  } = currentParent.context;
  const isEdit = !!name;

  return (
    <>
      {isEdit && url && (
        <NxReadOnly>
          <NxReadOnly.Label>{NUGET.SYMSRV_ENDPOINT.LABEL}</NxReadOnly.Label>
          <NxReadOnly.Data>{`${url}/symbols`}</NxReadOnly.Data>
        </NxReadOnly>
      )}

      <NxFormGroup
        label={NUGET.PROTOCOL_VERSION.LABEL}
        className="as-form-group-nuget-protocol-version"
        isRequired
      >
        <NxFormSelect
          {...FormUtils.selectProps('nugetProxy.nugetVersion', currentParent)}
          onChange={FormUtils.handleUpdate('nugetProxy.nugetVersion', sendParent)}
        >
          {Object.entries(NUGET_VERSIONS)?.map(([k, v]) => (
            <option key={v} value={k}>
              {v}
            </option>
          ))}
        </NxFormSelect>
      </NxFormGroup>

      <NxFormGroup
        label={NUGET.METADATA_QUERY_CACHE_AGE.LABEL}
        sublabel={NUGET.METADATA_QUERY_CACHE_AGE.SUBLABEL}
        isRequired
        className="as-form-group-nuget-cache"
      >
        <NxTextInput
          {...FormUtils.fieldProps('nugetProxy.queryCacheItemMaxAge', currentParent)}
          onChange={FormUtils.handleUpdate('nugetProxy.queryCacheItemMaxAge', sendParent)}
        />
      </NxFormGroup>

    </>
  );
}
