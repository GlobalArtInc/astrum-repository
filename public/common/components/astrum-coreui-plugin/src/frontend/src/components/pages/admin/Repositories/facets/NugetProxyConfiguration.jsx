
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsTextInput, AsFormSelect, AsReadOnly} from '@globalart/astrum-react-shared-components';

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
        <AsReadOnly>
          <AsReadOnly.Label>{NUGET.SYMSRV_ENDPOINT.LABEL}</AsReadOnly.Label>
          <AsReadOnly.Data>{`${url}/symbols`}</AsReadOnly.Data>
        </AsReadOnly>
      )}

      <AsFormGroup
        label={NUGET.PROTOCOL_VERSION.LABEL}
        className="as-form-group-nuget-protocol-version"
        isRequired
      >
        <AsFormSelect
          {...FormUtils.selectProps('nugetProxy.nugetVersion', currentParent)}
          onChange={FormUtils.handleUpdate('nugetProxy.nugetVersion', sendParent)}
        >
          {Object.entries(NUGET_VERSIONS)?.map(([k, v]) => (
            <option key={v} value={k}>
              {v}
            </option>
          ))}
        </AsFormSelect>
      </AsFormGroup>

      <AsFormGroup
        label={NUGET.METADATA_QUERY_CACHE_AGE.LABEL}
        sublabel={NUGET.METADATA_QUERY_CACHE_AGE.SUBLABEL}
        isRequired
        className="as-form-group-nuget-cache"
      >
        <AsTextInput
          {...FormUtils.fieldProps('nugetProxy.queryCacheItemMaxAge', currentParent)}
          onChange={FormUtils.handleUpdate('nugetProxy.queryCacheItemMaxAge', sendParent)}
        />
      </AsFormGroup>

    </>
  );
}
