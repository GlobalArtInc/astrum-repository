
import React from 'react';

import {ExtJS, FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsTextInput, AsCheckbox} from '@globalart/react-shared-components';

import FeatureFlags from '../../../../../constants/FeatureFlags';
import UIStrings from '../../../../../constants/UIStrings';

const {NUGET} = UIStrings.REPOSITORIES.EDITOR;
const {NUGET_SYMBOL_SERVER_ENABLED} = FeatureFlags;

export default function NugetSymbolServerConfiguration({parentMachine}) {
  const symbolServerEnabled = ExtJS.useState(() => ExtJS.state().getValue(NUGET_SYMBOL_SERVER_ENABLED));
  if (!symbolServerEnabled) {
    return null;
  }

  const [currentParent, sendParent] = parentMachine;

  return (
    <>
      <AsFormGroup
        label={NUGET.SYMBOL_SERVER_URL.LABEL}
        sublabel={NUGET.SYMBOL_SERVER_URL.SUBLABEL}
        className="as-form-group-nuget-symbol-server-url"
      >
        <AsTextInput
          {...FormUtils.fieldProps('nugetProxy.symbolServerUrl', currentParent)}
          onChange={FormUtils.handleUpdate('nugetProxy.symbolServerUrl', sendParent)}
        />
      </AsFormGroup>

      <AsFormGroup
        label={NUGET.ALLOW_ANONYMOUS_SYMBOL_ACCESS.LABEL}
        sublabel={NUGET.ALLOW_ANONYMOUS_SYMBOL_ACCESS.SUBLABEL}
      >
        <AsCheckbox
          {...FormUtils.checkboxProps('nugetProxy.allowAnonymousSymbolAccess', currentParent)}
          onChange={FormUtils.handleUpdate('nugetProxy.allowAnonymousSymbolAccess', sendParent)}
        >
          {NUGET.ALLOW_ANONYMOUS_SYMBOL_ACCESS.LABEL}
        </AsCheckbox>
      </AsFormGroup>
    </>
  );
}
