
import React from 'react';

import {AsReadOnly} from '@globalart/astrum-react-shared-components';

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
        <AsReadOnly>
          <AsReadOnly.Label>{NUGET.SYMSRV_ENDPOINT.LABEL}</AsReadOnly.Label>
          <AsReadOnly.Data>{`${url}/symbols`}</AsReadOnly.Data>
        </AsReadOnly>
      )}
    </>
  );
}
