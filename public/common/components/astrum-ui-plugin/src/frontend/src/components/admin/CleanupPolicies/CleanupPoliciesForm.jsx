
import React, {useCallback, useEffect} from 'react';
import {useMachine} from '@xstate/react';

import CheckboxControlledWrapper from '../../widgets/CheckboxControlledWrapper/CheckboxControlledWrapper';
import ExtJS from '../../../interface/ExtJS';
import FormUtils from '../../../interface/FormUtils';
import {
  AsButton,
  AsFontAwesomeIcon,
  AsFormSelect,
  AsStatefulForm,
  AsH2,
  AsP,
  AsFormGroup,
  AsTile,
  AsTextInput,
  AsFieldset
} from '@globalart/react-shared-components';

import { Page } from '../../layout';

import {
  ContentBody,
  PageHeader,
  PageTitle
} from '../../layout';

import CleanupPoliciesFormMachine from './CleanupPoliciesFormMachine';
import CleanupPoliciesPreview from './CleanupPoliciesPreview';
import CleanupPoliciesDryRun from './CleanupPoliciesDryRun';
import CleanupExclusionCriteria from './CleanupExclusionCriteria';

import UIStrings from '../../../constants/UIStrings';
import {faTrash} from '@fortawesome/free-solid-svg-icons';

import './CleanupPolicies.scss';
import {isEmpty} from 'ramda';
import {useCurrentStateAndParams, useRouter} from "@uirouter/react";
import {RouteNames} from "../../../constants/RouteNames";

const {CLEANUP_POLICIES: LABELS} = UIStrings;
const CLEANUP_POLICIES_ROUTES = RouteNames.ADMIN.REPOSITORY.CLEANUPPOLICIES;

export default function CleanupPoliciesForm() {
  const router = useRouter();
  const {state: routerState, params} = useCurrentStateAndParams();
  const onDone = useCallback(() => router.stateService.go(CLEANUP_POLICIES_ROUTES.LIST));
  const itemId = params?.itemId;
  const [state, send, actor] = useMachine(CleanupPoliciesFormMachine, {
    context: {
      pristineData: {
        name: itemId,
      },
    },

    actions: {
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone,
    },

    devTools: true,
  });

  useEffect(() => {
    // we should not render edit form if itemId is not provided
    if (routerState.name === CLEANUP_POLICIES_ROUTES.EDIT && !itemId) {
      router.stateService.go(RouteNames.MISSING_ROUTE);
    }
  }, [itemId]);


  const {
    pristineData,
    data,
    isTouched,
    loadError,
    criteriaByFormat,
    criteriaLastDownloadedEnabled,
    criteriaLastBlobUpdatedEnabled,
    criteriaAssetRegexEnabled,
    validationErrors: {criteriaSelected}
  } = state.context;

  const retainSupportedFormats = ['maven2' , 'docker'];

  const isEdit = Boolean(itemId);
  const isLoading = state.matches('loading');
  const hasData = !isEmpty(data);
  const isCriteriaFieldSetPristine = !isTouched.criteriaLastBlobUpdated &&
      !isTouched.criteriaLastDownloaded &&
      !isTouched.criteriaAssetRegex &&
      !isTouched.retain;
  const isPreviewEnabled =
    ExtJS.state().getValue('datastore.isPostgresql') &&
    ExtJS.state().getValue('astrum.cleanup.preview.enabled');

  const showRetainN = isRetainSupportedFormat(data.format) && isRetainEnabled(data.format);

  function isRetainEnabled(format) {
    return ExtJS.state()
        .getValue(`astrum.cleanup.${format}Retain`);
  }

  function isRetainSupportedFormat(format) {
    return retainSupportedFormats.includes(format);
  }

  function setCriteriaLastBlobUpdatedEnabled(checked) {
    send({type: 'SET_CRITERIA_LAST_BLOB_UPDATED_ENABLED', checked});
  }

  function setCriteriaLastDownloadedEnabled(checked) {
    send({type: 'SET_CRITERIA_LAST_DOWNLOADED_ENABLED', checked});
  }

  function setCriteriaAssetRegexEnabled(checked) {
    send({type: 'SET_CRITERIA_ASSET_REGEX_ENABLED', checked});
  }

  function confirmDelete() {
    send({type: 'CONFIRM_DELETE'});
  }

  function isAnyFieldApplicable() {
    return (
      isFieldApplicable('lastBlobUpdated') ||
      isFieldApplicable('lastDownloaded') ||
      isFieldApplicable('regex')
    );
  }

  function isFieldApplicable(fieldId) {
    return criteriaByFormat?.some(
      ({id, availableCriteria}) =>
        id === data.format && availableCriteria.includes(fieldId)
    );
  }

  function setReleaseType(value) {
    send({type: 'UPDATE_RELEASE_TYPE', value});
  }

  return (
    <Page className="as-cleanup-policies">
      <PageHeader>
        <PageTitle text={isEdit ? LABELS.EDIT_TITLE : LABELS.CREATE_TITLE} />
      </PageHeader>
      <ContentBody>
        <AsTile className="as-cleanup-policies-form">
          <AsH2>{LABELS.SUB_TITLE}</AsH2>
          <AsP>{LABELS.DESCRIPTION}</AsP>
          <AsStatefulForm
            {...FormUtils.formProps(state, send)}
            onCancel={onDone}
            additionalFooterBtns={
              itemId && (
                <AsButton
                  type="button"
                  variant="tertiary"
                  onClick={confirmDelete}
                >
                  <AsFontAwesomeIcon icon={faTrash} />
                  <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
                </AsButton>
              )
            }
          >
            {hasData && (
              <>
                <AsFormGroup
                  label={LABELS.NAME_LABEL}
                  sublabel={LABELS.NAME_DESCRIPTION}
                  isRequired
                >
                  <AsTextInput
                    {...FormUtils.fieldProps('name', state)}
                    disabled={pristineData.name}
                    onChange={FormUtils.handleUpdate('name', send)}
                    className="as-text-input--long"
                  />
                </AsFormGroup>
                <AsFormGroup
                  label={LABELS.FORMAT_LABEL}
                  sublabel={LABELS.FORMAT_DESCRIPTION}
                  isRequired
                >
                  <AsFormSelect
                    {...FormUtils.fieldProps('format', state)}
                    onChange={FormUtils.handleUpdate('format', send)}
                  >
                    <option value="">{LABELS.FORMAT_SELECT}</option>
                    {criteriaByFormat?.map((formatCriteria) => (
                      <option key={formatCriteria.id} value={formatCriteria.id}>
                        {formatCriteria.name}
                      </option>
                    ))}
                  </AsFormSelect>
                </AsFormGroup>
                <AsFormGroup label={LABELS.DESCRIPTION_LABEL}>
                  <AsTextInput
                    {...FormUtils.fieldProps('notes', state)}
                    onChange={FormUtils.handleUpdate('notes', send)}
                    className="as-text-input--long"
                    type="textarea"
                  />
                </AsFormGroup>
                {isFieldApplicable('isPrerelease') && (
                    <AsFormGroup
                        label={LABELS.RELEASE_TYPE_LABEL}
                        sublabel={LABELS.RELEASE_TYPE_SELECT}
                    >
                      <AsFormSelect
                          {...FormUtils.fieldProps(
                              'criteriaReleaseType',
                              state
                          )}
                          onChange={setReleaseType}
                          validatable
                          className="as-form-select--long"
                      >
                        {Object.keys(LABELS.RELEASE_TYPE).map((type) => {
                          const item = LABELS.RELEASE_TYPE[type];
                          return (
                              <option key={item.id} value={item.id}>
                                {item.label}
                              </option>
                          );
                        })}
                      </AsFormSelect>
                    </AsFormGroup>
                )}
                {isAnyFieldApplicable() && (
                  <>
                    <AsFieldset
                        className="criteria-fieldset"
                        label={LABELS.CRITERIA_LABEL}
                        sublabel={`${LABELS.CRITERIA_DESCRIPTION}. ${LABELS.MESSAGES.NO_CRITERIA_ERROR}`}
                        isPristine={isCriteriaFieldSetPristine}
                        validationErrors={criteriaSelected}
                        isRequired
                    >
                      {isFieldApplicable('lastBlobUpdated') && (
                          <CheckboxControlledWrapper
                              isChecked={Boolean(criteriaLastBlobUpdatedEnabled)}
                              onChange={setCriteriaLastBlobUpdatedEnabled}
                              id="criteria-last-blob-updated-group"
                              title={LABELS.LAST_UPDATED_CHECKBOX_TITLE(
                                  criteriaLastBlobUpdatedEnabled
                              )}
                          >
                            <AsFormGroup
                                label={LABELS.LAST_UPDATED_LABEL}
                                sublabel={LABELS.LAST_UPDATED_SUB_LABEL}
                                isRequired={criteriaLastBlobUpdatedEnabled}
                            >
                              <AsTextInput
                                  {...FormUtils.fieldProps(
                                      'criteriaLastBlobUpdated',
                                      state
                                  )}
                                  onChange={FormUtils.handleUpdate(
                                      'criteriaLastBlobUpdated',
                                      send
                                  )}
                                  placeholder={LABELS.PLACEHOLDER}
                                  disabled={!criteriaLastBlobUpdatedEnabled}
                                  className="as-text-input--short"
                              />
                            </AsFormGroup>
                          </CheckboxControlledWrapper>
                      )}
                      {isFieldApplicable('lastDownloaded') && (
                          <CheckboxControlledWrapper
                              isChecked={Boolean(criteriaLastDownloadedEnabled)}
                              onChange={setCriteriaLastDownloadedEnabled}
                              id="criteria-last-downloaded-group"
                              title={LABELS.LAST_DOWNLOADED_CHECKBOX_TITLE(
                                  criteriaLastDownloadedEnabled
                              )}
                          >
                            <AsFormGroup
                                label={LABELS.LAST_DOWNLOADED_LABEL}
                                isRequired={criteriaLastDownloadedEnabled}
                                sublabel={LABELS.LAST_DOWNLOADED_SUB_LABEL}
                            >
                              <AsTextInput
                                  {...FormUtils.fieldProps(
                                      'criteriaLastDownloaded',
                                      state
                                  )}
                                  onChange={FormUtils.handleUpdate(
                                      'criteriaLastDownloaded',
                                      send
                                  )}
                                  placeholder={LABELS.PLACEHOLDER}
                                  disabled={!criteriaLastDownloadedEnabled}
                                  className="as-text-input--short"
                              />
                            </AsFormGroup>
                          </CheckboxControlledWrapper>
                      )}
                      {isFieldApplicable('regex') && (
                          <CheckboxControlledWrapper
                              isChecked={Boolean(criteriaAssetRegexEnabled)}
                              onChange={setCriteriaAssetRegexEnabled}
                              id="criteria-asset-name-group"
                              title={LABELS.ASSET_NAME_CHECKBOX_TITLE(
                                  criteriaAssetRegexEnabled
                              )}
                          >
                            <AsFormGroup
                                label={LABELS.ASSET_NAME_LABEL}
                                sublabel={LABELS.ASSET_NAME_DESCRIPTION}
                                isRequired={criteriaAssetRegexEnabled}
                            >
                              <AsTextInput
                                  {...FormUtils.fieldProps(
                                      'criteriaAssetRegex',
                                      state
                                  )}
                                  onChange={FormUtils.handleUpdate(
                                      'criteriaAssetRegex',
                                      send
                                  )}
                                  disabled={!criteriaAssetRegexEnabled}
                                  className="as-text-input--long"
                              />
                            </AsFormGroup>
                          </CheckboxControlledWrapper>
                      )}
                      {showRetainN && (
                          <CleanupExclusionCriteria actor={actor}/>
                      )}
                    </AsFieldset>
                    {isPreviewEnabled && (
                      <CleanupPoliciesDryRun policyData={data} />
                    )}
                  </>
                )}
              </>
            )}
          </AsStatefulForm>
        </AsTile>

        {!isLoading && !loadError && hasData && !isPreviewEnabled && (
          <CleanupPoliciesPreview policyData={data} />
        )}
      </ContentBody>
    </Page>
  );
}
