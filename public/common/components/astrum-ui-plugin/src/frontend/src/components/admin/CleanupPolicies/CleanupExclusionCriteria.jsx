

import React from 'react';
import {NxCheckbox, NxFormGroup, NxInfoAlert, NxTextInput} from '@globalart/react-shared-components';
import {isReleaseType} from './CleanupPoliciesHelper';
import ExtJS from '../../../interface/ExtJS';
import FormUtils from '../../../interface/FormUtils';
import {useActor} from '@xstate/react';
import UIStrings from '../../../constants/UIStrings';

const {CLEANUP_POLICIES: LABELS} = UIStrings;

export default function CleanupExclusionCriteria({actor}) {
  const [state, send] = useActor(actor);

  const {
    criteriaByFormat,
    data:
    {
      format,
      criteriaReleaseType,
      criteriaLastDownloaded,
      criteriaLastBlobUpdated,
      criteriaAssetRegex
    },
    exclusionCriteriaEnabled,
    exclusionSortBy,
  } = state.context;

  const criteriaSelected = (criteriaLastDownloaded || criteriaLastBlobUpdated || criteriaAssetRegex);

  const isNormalizedVersionTaskDone = ExtJS.state().getValue(
      `${format}.normalized.version.available`
  );

  const isPreRelease = () => criteriaByFormat?.some(
      ({id, availableCriteria}) => id === format && availableCriteria.includes("isPrerelease"));

  const setExclusionCriteriaEnabled = (checked) => {
    if (isNormalizedVersionTaskDone) {
      send({type: 'SET_EXCLUSION_CRITERIA_ENABLED', checked});
    }
  }

  const setRetainValue = (value) => {
    send({type: 'UPDATE_RETAIN', name: 'retain', value });
  }

  return (
      <>
        {!isNormalizedVersionTaskDone && (
            <NxInfoAlert className="retain-n__alert">
              {LABELS.EXCLUSION_CRITERIA.NORMALIZED_VERSION_ALERT}
            </NxInfoAlert>
        )}
        {((isPreRelease() && !isReleaseType(criteriaReleaseType)) &&
            isNormalizedVersionTaskDone) && (
            <NxInfoAlert className="retain-n__alert">
              {LABELS.EXCLUSION_CRITERIA.ALERT}
            </NxInfoAlert>
        )}
        <NxCheckbox
            isChecked={Boolean(exclusionCriteriaEnabled)}
            onChange={setExclusionCriteriaEnabled}
            disabled={
              !isNormalizedVersionTaskDone ||
              (isPreRelease() && !isReleaseType(criteriaReleaseType)) ||
              !criteriaSelected
            }
            className="retain-n__checkbox"
        >
          <b>{LABELS.EXCLUSION_CRITERIA.LABEL}</b>
        </NxCheckbox>
        {(isReleaseType(criteriaReleaseType) || format=='docker') && !criteriaSelected && (
            <NxInfoAlert className="retain-n-criteria__alert">
              {LABELS.EXCLUSION_CRITERIA.ADDITIONAL_CRITERIA_ALERT}
            </NxInfoAlert>
        )}
        <NxFormGroup
            label={LABELS.EXCLUSION_CRITERIA.VERSION_LABEL}
            isRequired={exclusionCriteriaEnabled}
            sublabel={LABELS.EXCLUSION_CRITERIA.SUB_LABEL(exclusionSortBy, LABELS.EXCLUSION_CRITERIA.SORT_BY)}
            className="retain-n__group"
        >
          <NxTextInput
              className="nx-text-input--short"
              disabled={!exclusionCriteriaEnabled}
              {...FormUtils.fieldProps('retain', state)}
              onChange={setRetainValue}
          />
        </NxFormGroup>
      </>
  );
}
