
import React from 'react';

import {NxReadOnly} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {NUGET} = UIStrings.REPOSITORIES.EDITOR;

export default function NugetHostedConfiguration({parentMachine}) {
  const [currentParent] = parentMachine;

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
    </>
  );
}
