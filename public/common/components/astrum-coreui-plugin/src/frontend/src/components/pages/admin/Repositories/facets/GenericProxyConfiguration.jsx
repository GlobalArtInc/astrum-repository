
import React from 'react';

import {ExtJS, FormUtils, Textfield, UseAstrumTruststore} from '@globalart/astrum-ui-plugin';

import {
  NxCheckbox,
  NxFieldset,
  NxFormGroup,
  NxTextInput,
} from '@globalart/react-shared-components';


import UIStrings from '../../../../../constants/UIStrings';

import DockerIndexConfiguration from './DockerIndexConfiguration';
import DockerForeignLayerConfiguration from './DockerForeignLayerConfiguration';

const {EDITOR} = UIStrings.REPOSITORIES;

const REPLICATION_FEATURE = 'replicationFeatureEnabled';
const REPLICATION_FORMATS = 'replicationSupportedFormats';

export default function GenericProxyConfiguration({parentMachine}) {
  const [parentState, sendParent] = parentMachine;
  const {
    data: {
      format,
      type,
      replication,
      proxy: {remoteUrl}
    }
  } = parentState.context;


  const isReplicationEnabled = ExtJS.state().getValue(REPLICATION_FEATURE) || false;
  const preemptivePullEnabled = replication?.preemptivePullEnabled || false;
  const formatReplicationSupported = ((ExtJS.state().getValue(REPLICATION_FORMATS) instanceof Array) &&
      ExtJS.state().getValue(REPLICATION_FORMATS).includes(format)) || false;

  function setPreemptivePullEnabled(event) {
    sendParent({
      type: 'UPDATE_PREEMPTIVE_PULL',
      checked: event.currentTarget.checked
    });
  }

  return (
    <>
      <h2 className="nx-h2">{EDITOR.PROXY_CAPTION}</h2>

      <NxFormGroup
        label={EDITOR.REMOTE_STORAGE_LABEL}
        sublabel={
          EDITOR.REMOTE_STORAGE_SUBLABEL +
          (EDITOR.REMOTE_URL_EXAMPLES[format] || EDITOR.REMOTE_URL_EXAMPLES.default)
        }
        isRequired
        className="as-form-group-remote-storage"
      >
        <NxTextInput
          {...FormUtils.fieldProps('proxy.remoteUrl', parentState)}
          onChange={FormUtils.handleUpdate('proxy.remoteUrl', sendParent)}
          placeholder={EDITOR.URL_PLACEHOLDER}
        />
      </NxFormGroup>

      <UseAstrumTruststore
        remoteUrl={remoteUrl}
        {...FormUtils.checkboxProps('httpClient.connection.useTrustStore', parentState)}
        onChange={FormUtils.handleUpdate('httpClient.connection.useTrustStore', sendParent)}
      />

      {format === 'docker' && <DockerIndexConfiguration parentMachine={parentMachine} />}

      {format === 'docker' && type === 'proxy' && <DockerForeignLayerConfiguration parentMachine={parentMachine} />}

      {isReplicationEnabled && formatReplicationSupported && (
        <>
          <NxFormGroup
            label={EDITOR.PREEMPTIVE_PULL_LABEL}
            sublabel={EDITOR.PREEMPTIVE_PULL_SUBLABEL}
          >
            <NxCheckbox
              isChecked={Boolean(preemptivePullEnabled)}
              onChange={setPreemptivePullEnabled}
            >
              {EDITOR.ENABLED_CHECKBOX_DESCR}
            </NxCheckbox>
          </NxFormGroup>

          <NxFormGroup
            label={EDITOR.ASSET_NAME_LABEL}
            sublabel={EDITOR.ASSET_NAME_DESCRIPTION}
            className="as-form-group-asset-matcher"
          >
            <Textfield
              {...FormUtils.fieldProps('replication.assetPathRegex', parentState)}
              onChange={FormUtils.handleUpdate('replication.assetPathRegex', sendParent)}
              disabled={!Boolean(preemptivePullEnabled)}
            />
          </NxFormGroup>
        </>
      )}

      <NxFormGroup
        label={EDITOR.PRESERVE_ENCODED_CHARS_LABEL}
        sublabel={EDITOR.PRESERVE_ENCODED_CHARS_SUBLABEL}
      >
        <NxCheckbox
          {...FormUtils.checkboxProps('proxy.preserveEncodedCharacters', parentState)}
          onChange={FormUtils.handleUpdate('proxy.preserveEncodedCharacters', sendParent)}
        >
          {EDITOR.ENABLED_CHECKBOX_DESCR}
        </NxCheckbox>
      </NxFormGroup>

      <NxFieldset label={EDITOR.BLOCKING_LABEL} className="as-form-group-proxy-blocking">
        <NxCheckbox
          {...FormUtils.checkboxProps('httpClient.blocked', parentState)}
          onChange={FormUtils.handleUpdate('httpClient.blocked', sendParent)}
        >
          {EDITOR.BLOCK_DESCR}
        </NxCheckbox>
        <NxCheckbox
          {...FormUtils.checkboxProps('httpClient.autoBlock', parentState)}
          onChange={FormUtils.handleUpdate('httpClient.autoBlock', sendParent)}
        >
          {EDITOR.AUTO_BLOCK_DESCR}
        </NxCheckbox>
      </NxFieldset>

      <NxFormGroup
        label={EDITOR.MAX_COMP_AGE_LABEL}
        sublabel={EDITOR.MAX_COMP_AGE_SUBLABEL}
        isRequired
        className="as-form-group-max-component-age"
      >
        <NxTextInput
          {...FormUtils.fieldProps('proxy.contentMaxAge', parentState)}
          onChange={FormUtils.handleUpdate('proxy.contentMaxAge', sendParent)}
          className="nx-text-input--short"
        />
      </NxFormGroup>

      <NxFormGroup
        label={EDITOR.MAX_META_AGE_LABEL}
        sublabel={EDITOR.MAX_META_AGE_SUBLABEL}
        isRequired
        className="as-form-group-max-metadata-age"
      >
        <NxTextInput
          {...FormUtils.fieldProps('proxy.metadataMaxAge', parentState)}
          onChange={FormUtils.handleUpdate('proxy.metadataMaxAge', sendParent)}
          className="nx-text-input--short"
        />
      </NxFormGroup>
    </>
  );
}
