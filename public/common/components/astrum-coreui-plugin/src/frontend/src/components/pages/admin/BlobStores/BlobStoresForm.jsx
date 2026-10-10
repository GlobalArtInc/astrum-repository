
import React, {useCallback, useEffect} from 'react';
import {useMachine} from '@xstate/react';
import {path} from 'ramda';
import {faTrash} from "@fortawesome/free-solid-svg-icons";

import {DynamicFormField, ExtJS, FormUtils, Permissions} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsCheckbox,
  AsFieldset,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsFormSelect,
  AsInfoAlert,
  AsP,
  AsStatefulForm,
  AsTextInput,
  AsTooltip,
  AsWarningAlert
} from '@globalart/react-shared-components';

import {
  Page,
  PageActions,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';

import BlobStoresFormMachine, {canUseSpaceUsedQuotaOnly, SPACE_USED_QUOTA_ID} from './BlobStoresFormMachine';
import UIStrings from '../../../../constants/UIStrings';
import CustomBlobStoreSettings from './CustomBlobStoreSettings';
import BlobStoreWarning from './BlobStoreWarning';
import BlobStoresConvertModal from './BlobStoresConvertModal';
import {useCurrentStateAndParams, useRouter} from "@uirouter/react";
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';
import './BlobStores.scss';

const {BLOB_STORES: {FORM}} = UIStrings;

const ADMIN = ROUTE_NAMES.ADMIN;

export default function BlobStoresForm() {
  const router = useRouter();
  const {state: routerState, params} = useCurrentStateAndParams();
  const onDone = useCallback(() => router.stateService.go(ADMIN.REPOSITORY.BLOBSTORES.LIST));
  const pristineData = params.type && params.name ? {
    type: decodeURIComponent(params.type),
    name: decodeURIComponent(params.name)
  } : {};

  const hasUpdatePermissions = ExtJS.checkPermission(Permissions.BLOB_STORES.UPDATE);
  const hasDeletePermission = ExtJS.checkPermission(Permissions.BLOB_STORES.DELETE);

  useEffect(() => {
    // we should not render edit form if type or name are not provided
    if (routerState.name === ADMIN.REPOSITORY.BLOBSTORES.EDIT && (!params.type || !params.name)) {
      router.stateService.go(ROUTE_NAMES.MISSING_ROUTE);
    }
  }, [params.type, params.name, routerState.name]);

  const [current, send, service] = useMachine(BlobStoresFormMachine, {
    context: {
      pristineData
    },

    actions: {
      onCancel: onDone,
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone
    },
    devTools: true
  });

  const showConvertToGroupModal = current.matches('modalConvertToGroup');
  const isCreate = routerState.name === ADMIN.REPOSITORY.BLOBSTORES.CREATE;
  const isEdit = !isCreate;
  const {
    blobStoreUsage,
    data,
    quotaTypes,
    repositoryUsage,
    type,
    types,
    bucketRegionMismatch,
    bucketEncryptionMismatch
  } = current.context;
  const hasSoftQuota = path(['softQuota', 'enabled'], data);
  const cannotDelete = blobStoreUsage > 0 || repositoryUsage > 0 || !hasDeletePermission;
  const deleteTooltip = cannotDelete ?
      UIStrings.BLOB_STORES.MESSAGES.CANNOT_DELETE(repositoryUsage, blobStoreUsage, hasDeletePermission) :
      null;
  const isTypeSelected = Boolean(type)

  const setType = (value) => send({type: 'SET_TYPE', value});

  const updateDynamicField = (name, value) => {
    send({
      type: 'UPDATE',
      data: {
        [name]: value
      }
    });
  }

  const toggleSoftQuota = (value) => {
    send({type: 'UPDATE_SOFT_QUOTA', name: 'enabled', value, data});
  }

  const updateSoftQuotaType = (value) => send({ type: 'UPDATE_SOFT_QUOTA', name: 'type', value});
  const updateSoftQuotaLimit = (value) => send({ type: 'UPDATE_SOFT_QUOTA', name: 'limit', value});

  const confirmDelete = () => send({type: 'CONFIRM_DELETE'});
  const modalConvertToGroupOpen = () => send({type: 'MODAL_CONVERT_TO_GROUP_OPEN'});
  const modalConvertToGroupClose = () => send({type: 'MODAL_CONVERT_TO_GROUP_CLOSE'});

  const spaceUsedQuotaName = quotaTypes?.find((it) => it.id === SPACE_USED_QUOTA_ID).name;

  function getErrorTitleMessage() {
    if (bucketRegionMismatch) {
      return UIStrings.BLOB_STORES.GOOGLE.ERROR.bucketRegionMismatchTitle
    }
    if (bucketEncryptionMismatch) {
      return UIStrings.BLOB_STORES.GOOGLE.ERROR.bucketEncryptionMismatchTitle
    }
    return undefined;
  }

  const formProps = FormUtils.formProps(current, send);
  formProps.onSubmit = hasUpdatePermissions ? formProps.onSubmit : () => {};
  formProps.validationErrors = hasUpdatePermissions ? formProps.validationErrors : null;

  return <Page className="as-blob-stores">
    <PageHeader>
      <PageTitle text={isEdit ? FORM.EDIT_TILE(pristineData.name) : FORM.CREATE_TITLE}
                 description={isEdit ? FORM.EDIT_DESCRIPTION(type?.name || pristineData.type) : null}/>
      {isEdit && type?.id !== 'group' && types?.some(type => type.id === 'group') &&
      <PageActions>
        <AsButton variant="primary" onClick={modalConvertToGroupOpen} disabled={!hasUpdatePermissions} data-analytics-id="as-blobstore-convert-to-group">{FORM.CONVERT_TO_GROUP_BUTTON}</AsButton>
      </PageActions>
      }
    </PageHeader>
    <Section>
      <AsStatefulForm className="as-blob-stores-form"
        {...formProps}
        submitErrorTitleMessage={getErrorTitleMessage()}
        onCancel={onDone}
        submitBtnClasses={!hasUpdatePermissions && 'disabled'}
        additionalFooterBtns={isEdit &&
          <AsTooltip title={deleteTooltip}>
            <AsButton variant="tertiary" className={cannotDelete && 'disabled'} onClick={confirmDelete} type="button" data-analytics-id="as-blobstore-delete">
              <AsFontAwesomeIcon icon={faTrash}/>
              <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
            </AsButton>
          </AsTooltip>
        }
      >
        {isEdit && hasUpdatePermissions && <AsInfoAlert>{FORM.EDIT_WARNING}</AsInfoAlert>}
        {isEdit && !hasUpdatePermissions && <AsWarningAlert>{FORM.NO_PERMISSION_WARNING}</AsWarningAlert>}
        {isCreate &&
        <AsFormGroup label={FORM.TYPE.label} sublabel={FORM.TYPE.sublabel} isRequired>
          <AsFormSelect {...FormUtils.selectProps('type', current)} value={type?.id} onChange={setType}>
            <option disabled={isTypeSelected} value=""></option>
            {types?.map(({id, name}) => <option key={id} value={id}>{name}</option>)}
          </AsFormSelect>
        </AsFormGroup>
        }
        {isTypeSelected &&
        <>
          {(isCreate || (isEdit && hasUpdatePermissions)) && <BlobStoreWarning type={type} service={service} />}
          {isCreate &&
              <AsFormGroup
                  className="blob-store-name"
                  label={FORM.NAME.label}
                  isRequired
              >
                <AsTextInput
                    {...FormUtils.fieldProps('name', current)}
                    onChange={FormUtils.handleUpdate('name', send)}
                />
              </AsFormGroup>
          }
          <CustomBlobStoreSettings type={type} service={service}/>
          {type?.fields?.map(field =>
              <DynamicFormField
                  key={field.id}
                  id={field.id}
                  current={current}
                  initialValue={field.initialValue}
                  onChange={updateDynamicField}
                  dynamicProps={field} />
          )}
          <div className="as-soft-quota">
            <AsFieldset label={FORM.SOFT_QUOTA.ENABLED.label} sublabel={FORM.SOFT_QUOTA.ENABLED.sublabel}>
              <AsCheckbox {...FormUtils.checkboxProps(['softQuota', 'enabled'], current)} onChange={toggleSoftQuota}>
                {FORM.SOFT_QUOTA.ENABLED.text}
              </AsCheckbox>
            </AsFieldset>

            {hasSoftQuota &&
            <>
              <AsFormGroup label={FORM.SOFT_QUOTA.TYPE.label} isRequired>
                {canUseSpaceUsedQuotaOnly(type)
                  ? <AsP>{spaceUsedQuotaName}</AsP>
                  : <AsFormSelect {...FormUtils.fieldProps(['softQuota', 'type'], current)} validatable onChange={updateSoftQuotaType}>
                      <option value="" disabled></option>
                      {quotaTypes.map(({ id, name }) => <option key={id} value={id}>{name}</option>)}
                    </AsFormSelect>
                }
              </AsFormGroup>
              <AsFormGroup label={FORM.SOFT_QUOTA.LIMIT.label} isRequired>
                <AsTextInput {...FormUtils.fieldProps(['softQuota', 'limit'], current)} onChange={updateSoftQuotaLimit}/>
              </AsFormGroup>
            </>
            }
          </div>
        </>
        }
      </AsStatefulForm>
    </Section>
    {showConvertToGroupModal &&
        <BlobStoresConvertModal
            name={data.name}
            onDone={onDone}
            onCancel={modalConvertToGroupClose}
        />
    }
  </Page>;
}
