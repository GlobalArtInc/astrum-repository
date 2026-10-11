import React, {useEffect, useRef, useState} from 'react';
import Axios from 'axios';

import {
  AsButton,
  AsErrorAlert,
  AsInfoAlert,
  AsLoadingSpinner,
  AsTooltip,
  AsWarningAlert
} from '@globalart/astrum-react-shared-components';

import HumanReadableUtils from '../../../interface/HumanReadableUtils';
import ValidationUtils from '../../../interface/ValidationUtils';
import UIStrings from '../../../constants/UIStrings';
import {URL, previewErrorMessage, previewRequest} from './CleanupPoliciesHelper';

const {SUMMARY} = UIStrings.CLEANUP_POLICIES;

/**
 * Counts, on request, everything the policy being edited would delete from the selected repository.
 */
export default function CleanupPolicySummary({policyData, repository}) {
  const [state, setState] = useState({status: 'idle'});
  const latestRequest = useRef(0);

  const {
    criteriaLastBlobUpdated,
    criteriaLastDownloaded,
    criteriaReleaseType,
    criteriaAssetRegex,
    retain,
    sortBy
  } = policyData;
  const unavailable = ValidationUtils.isBlank(repository) || (
      ValidationUtils.isBlank(criteriaLastBlobUpdated) &&
      ValidationUtils.isBlank(criteriaLastDownloaded) &&
      ValidationUtils.isBlank(criteriaReleaseType) &&
      ValidationUtils.isBlank(criteriaAssetRegex)
  );
  const isLoading = state.status === 'loading';

  useEffect(() => {
    latestRequest.current += 1;
    setState({status: 'idle'});
  }, [repository, criteriaLastBlobUpdated, criteriaLastDownloaded, criteriaReleaseType, criteriaAssetRegex, retain,
    sortBy]);

  async function calculate() {
    if (unavailable || isLoading) {
      return;
    }
    const request = ++latestRequest.current;
    setState({status: 'loading'});
    try {
      const {data} = await Axios.post(URL.previewSummaryUrl, previewRequest(repository, policyData));
      if (request === latestRequest.current) {
        setState({status: 'done', summary: data});
      }
    }
    catch (error) {
      if (request === latestRequest.current) {
        setState({status: 'error', error: previewErrorMessage(error) || SUMMARY.ERROR});
      }
    }
  }

  return <div className="as-cleanup-policies-summary" aria-live="polite">
    <div className="as-cleanup-policies-summary__header">
      <div>
        <h3 className="as-h3 as-cleanup-policies-summary__title">{SUMMARY.TITLE}</h3>
        <p className="as-p as-cleanup-policies-summary__description">{SUMMARY.DESCRIPTION}</p>
      </div>
      <AsTooltip title={unavailable ? UIStrings.CLEANUP_POLICIES.PREVIEW.BUTTON_TOOLTIP : undefined}>
        <AsButton type="button"
                  className={unavailable || isLoading ? 'disabled' : ''}
                  aria-disabled={unavailable || isLoading}
                  onClick={calculate}>
          {SUMMARY.BUTTON}
        </AsButton>
      </AsTooltip>
    </div>
    {isLoading && <AsLoadingSpinner>{SUMMARY.CALCULATING}</AsLoadingSpinner>}
    {state.status === 'error' && <AsErrorAlert>{state.error}</AsErrorAlert>}
    {state.status === 'done' && <SummaryResult summary={state.summary}/>}
  </div>;
}

function SummaryResult({summary}) {
  const {repository, componentCount, assetCount, totalSize, complete} = summary;

  if (complete && componentCount === 0) {
    return <AsInfoAlert>{SUMMARY.EMPTY(repository)}</AsInfoAlert>;
  }

  return <div className="as-cleanup-policies-summary__result">
    <p className="as-p">{complete ? SUMMARY.RESULT(repository) : SUMMARY.RESULT_AT_LEAST(repository)}</p>
    <dl className="as-cleanup-policies-summary__figures">
      <div>
        <dt>{SUMMARY.COMPONENTS}</dt>
        <dd>{componentCount.toLocaleString()}</dd>
      </div>
      <div>
        <dt>{SUMMARY.FILES}</dt>
        <dd>{assetCount.toLocaleString()}</dd>
      </div>
      <div>
        <dt>{SUMMARY.SIZE}</dt>
        <dd>{HumanReadableUtils.bytesToString(totalSize)}</dd>
      </div>
    </dl>
    {!complete && <AsWarningAlert>{SUMMARY.INCOMPLETE}</AsWarningAlert>}
  </div>;
}
