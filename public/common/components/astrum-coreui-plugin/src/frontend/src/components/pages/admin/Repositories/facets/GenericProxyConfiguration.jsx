
import React from 'react';

import {ExtJS, FormUtils, Textfield, UseAstrumTruststore} from '@globalart/astrum-ui-plugin';

import {
  AsCheckbox,
  AsFieldset,
  AsFormGroup,
  AsTextInput,
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
      <h2 className="as-h2">{EDITOR.PROXY_CAPTION}</h2>

      <AsFormGroup
        label={EDITOR.REMOTE_STORAGE_LABEL}
        sublabel={
          EDITOR.REMOTE_STORAGE_SUBLABEL +
          (EDITOR.REMOTE_URL_EXAMPLES[format] || EDITOR.REMOTE_URL_EXAMPLES.default)
        }
        isRequired
        className="as-form-group-remote-storage"
      >
        <AsTextInput
          {...FormUtils.fieldProps('proxy.remoteUrl', parentState)}
          onChange={FormUtils.handleUpdate('proxy.remoteUrl', sendParent)}
          placeholder={EDITOR.URL_PLACEHOLDER}
        />
      </AsFormGroup>

      <UseAstrumTruststore
        remoteUrl={remoteUrl}
        {...FormUtils.checkboxProps('httpClient.connection.useTrustStore', parentState)}
        onChange={FormUtils.handleUpdate('httpClient.connection.useTrustStore', sendParent)}
      />

      {format === 'docker' && <DockerIndexConfiguration parentMachine={parentMachine} />}

      {format === 'docker' && type === 'proxy' && <DockerForeignLayerConfiguration parentMachine={parentMachine} />}

      {isReplicationEnabled && formatReplicationSupported && (
        <>
          <AsFormGroup
            label={EDITOR.PREEMPTIVE_PULL_LABEL}
            sublabel={EDITOR.PREEMPTIVE_PULL_SUBLABEL}
          >
            <AsCheckbox
              isChecked={Boolean(preemptivePullEnabled)}
              onChange={setPreemptivePullEnabled}
            >
              {EDITOR.ENABLED_CHECKBOX_DESCR}
            </AsCheckbox>
          </AsFormGroup>

          <AsFormGroup
            label={EDITOR.ASSET_NAME_LABEL}
            sublabel={EDITOR.ASSET_NAME_DESCRIPTION}
            className="as-form-group-asset-matcher"
          >
            <Textfield
              {...FormUtils.fieldProps('replication.assetPathRegex', parentState)}
              onChange={FormUtils.handleUpdate('replication.assetPathRegex', sendParent)}
              disabled={!Boolean(preemptivePullEnabled)}
            />
          </AsFormGroup>
        </>
      )}

      <AsFormGroup
        label={EDITOR.PRESERVE_ENCODED_CHARS_LABEL}
        sublabel={EDITOR.PRESERVE_ENCODED_CHARS_SUBLABEL}
      >
        <AsCheckbox
          {...FormUtils.checkboxProps('proxy.preserveEncodedCharacters', parentState)}
          onChange={FormUtils.handleUpdate('proxy.preserveEncodedCharacters', sendParent)}
        >
          {EDITOR.ENABLED_CHECKBOX_DESCR}
        </AsCheckbox>
      </AsFormGroup>

      <AsFieldset label={EDITOR.BLOCKING_LABEL} className="as-form-group-proxy-blocking">
        <AsCheckbox
          {...FormUtils.checkboxProps('httpClient.blocked', parentState)}
          onChange={FormUtils.handleUpdate('httpClient.blocked', sendParent)}
        >
          {EDITOR.BLOCK_DESCR}
        </AsCheckbox>
        <AsCheckbox
          {...FormUtils.checkboxProps('httpClient.autoBlock', parentState)}
          onChange={FormUtils.handleUpdate('httpClient.autoBlock', sendParent)}
        >
          {EDITOR.AUTO_BLOCK_DESCR}
        </AsCheckbox>
      </AsFieldset>

      <AsFormGroup
        label={EDITOR.MAX_COMP_AGE_LABEL}
        sublabel={EDITOR.MAX_COMP_AGE_SUBLABEL}
        isRequired
        className="as-form-group-max-component-age"
      >
        <AsTextInput
          {...FormUtils.fieldProps('proxy.contentMaxAge', parentState)}
          onChange={FormUtils.handleUpdate('proxy.contentMaxAge', sendParent)}
          className="as-text-input--short"
        />
      </AsFormGroup>

      <AsFormGroup
        label={EDITOR.MAX_META_AGE_LABEL}
        sublabel={EDITOR.MAX_META_AGE_SUBLABEL}
        isRequired
        className="as-form-group-max-metadata-age"
      >
        <AsTextInput
          {...FormUtils.fieldProps('proxy.metadataMaxAge', parentState)}
          onChange={FormUtils.handleUpdate('proxy.metadataMaxAge', sendParent)}
          className="as-text-input--short"
        />
      </AsFormGroup>
    </>
  );
}
