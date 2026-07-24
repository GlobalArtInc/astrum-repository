
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxCheckbox, NxFieldset, NxFormSelect} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';
import DockerRedeployLatesConfiguration from './DockerRedeployLatesConfiguration';

const {EDITOR} = UIStrings.REPOSITORIES;

const deploymentPolicies = Object.entries(EDITOR.DEPLOYMENT_POLICY_OPTIONS);

    // Formats that do not support namespace confusion protection via Firewall
const UNSUPPORTED_NAMESPACE_CONFUSION_FORMATS = ['docker'];

export default function GenericHostedConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;

  const {format, type} = parentState.context.data;

  const supportsNamespaceConfusion = !UNSUPPORTED_NAMESPACE_CONFUSION_FORMATS.includes(format);

  return (
    <>
      <h2 className="nx-h2">{EDITOR.HOSTED_CAPTION}</h2>

      <NxFormGroup
        label={EDITOR.DEPLOYMENT_POLICY_LABEL}
        sublabel={EDITOR.DEPLOYMENT_POLICY_SUBLABEL}
        isRequired
        className="as-form-group-hosted"
      >
        <NxFormSelect
          {...FormUtils.selectProps('storage.writePolicy', parentState)}
          onChange={FormUtils.handleUpdate('storage.writePolicy', sendParent)}
        >
          {deploymentPolicies?.map(([value, displayName]) => (
            <option key={value} value={value}>
              {displayName}
            </option>
          ))}
        </NxFormSelect>
      </NxFormGroup>

      {supportsNamespaceConfusion && (
        <NxFieldset
          label={EDITOR.PROPRIETARY_COMPONENTS_LABEL}
          className="as-form-group-is-proprietary"
        >
          <NxCheckbox
            {...FormUtils.checkboxProps('component.proprietaryComponents', parentState)}
            onChange={FormUtils.handleUpdate('component.proprietaryComponents', sendParent)}
          >
            {EDITOR.PROPRIETARY_COMPONENTS_DESCR}
          </NxCheckbox>
        </NxFieldset>
      )}

      {format === 'docker' && <DockerRedeployLatesConfiguration parentMachine={parentMachine} />}
    </>
  );
}
