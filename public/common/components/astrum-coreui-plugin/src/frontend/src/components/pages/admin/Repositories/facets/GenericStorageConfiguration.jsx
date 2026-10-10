
import React, {useEffect} from 'react';

import {ExtAPIUtils, FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsFormGroup,
  AsCheckbox,
  AsLoadWrapper,
  AsFieldset,
  AsFormSelect
} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function GenericStorageConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;
  const [blobStoresState, blobStoresSend] = ExtAPIUtils.useExtMachine('coreui_Blobstore', 'readNames');

  const {data: blobStores, error} = blobStoresState.context;
  const {format, type} = currentParent.context.data;
  const loadBlobStores = () => blobStoresSend({type: 'LOAD'});
  const isLoading = blobStoresState.matches('loading');

  const {name} = currentParent.context.pristineData;
  const isEdit = !!name;

  useEffect(() => {
    loadBlobStores();
  }, [format, type]);

  useEffect(() => {
    sendParent({
      type: 'SET_DEFAULT_BLOB_STORE',
      name: 'storage.blobStoreName',
      value: getDefaultBlobStore(blobStores)
    });
  }, [blobStores]);

  return (
    <>
      <h2 className="as-h2">{EDITOR.STORAGE_CAPTION}</h2>
      <AsLoadWrapper loading={isLoading} error={error} retryHandler={loadBlobStores}>
        <AsFormGroup label={EDITOR.BLOB_STORE_LABEL} isRequired className="as-form-group-store">
          <AsFormSelect
            {...FormUtils.selectProps('storage.blobStoreName', currentParent)}
            onChange={FormUtils.handleUpdate('storage.blobStoreName', sendParent)}
            disabled={isEdit}
          >
            <option value="">{EDITOR.SELECT_STORE_OPTION}</option>
            {blobStores?.map(({name}) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </AsFormSelect>
        </AsFormGroup>

        {type !== 'group' && (
          <AsFieldset
            label={EDITOR.CONTENT_VALIDATION_LABEL}
            className="as-form-group-content-validation"
          >
            <AsCheckbox
              {...FormUtils.checkboxProps('storage.strictContentTypeValidation', currentParent)}
              onChange={FormUtils.handleUpdate('storage.strictContentTypeValidation', sendParent)}
            >
              {EDITOR.ENABLED_CHECKBOX_DESCR}
            </AsCheckbox>
          </AsFieldset>
        )}
      </AsLoadWrapper>
    </>
  );
}

const getDefaultBlobStore = (blobStores) =>
  blobStores?.length && blobStores.length === 1 ? blobStores[0].name : '';
