import React, { useEffect, useRef, useState } from 'react';
import { Box, Flex, Text, Tooltip } from '@radix-ui/themes';
import { Calculator } from 'lucide-react';

import { SettingsButton, SettingsAlert } from '../../../../shared/form';
import UIStrings from '../../../../../../constants/UIStrings';
import HumanReadableUtils from '../../../../../../interface/HumanReadableUtils';
import { useCleanupPoliciesApi } from './useCleanupPoliciesApi';
import { CleanupPolicyFormData, CleanupPreviewSummary } from './types';

import './CleanupPolicySummary.scss';

const { SUMMARY, PREVIEW } = UIStrings.CLEANUP_POLICIES;

interface CleanupPolicySummaryProps {
  policyData: CleanupPolicyFormData;
  repository: string;
  policyName?: string;
}

type SummaryState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'done'; summary: CleanupPreviewSummary }
  | { status: 'error'; error: string };

/**
 * CleanupPolicySummary - Count, on request, everything the policy would delete from the selected repository
 */
export function CleanupPolicySummary({ policyData, repository, policyName }: CleanupPolicySummaryProps) {
  const [state, setState] = useState<SummaryState>({ status: 'idle' });
  const latestRequest = useRef(0);
  const { fetchCleanupSummary } = useCleanupPoliciesApi();

  const available =
    Boolean(repository) &&
    (Boolean(policyData.criteriaLastBlobUpdated) ||
      Boolean(policyData.criteriaLastDownloaded) ||
      Boolean(policyData.criteriaReleaseType) ||
      Boolean(policyData.criteriaAssetRegex));
  const isLoading = state.status === 'loading';

  useEffect(() => {
    latestRequest.current += 1;
    setState({ status: 'idle' });
  }, [
    repository,
    policyData.criteriaLastBlobUpdated,
    policyData.criteriaLastDownloaded,
    policyData.criteriaReleaseType,
    policyData.criteriaAssetRegex,
    policyData.retain,
    policyData.sortBy,
  ]);

  const calculate = async () => {
    if (!available || isLoading) return;

    const request = ++latestRequest.current;
    setState({ status: 'loading' });
    try {
      const summary = await fetchCleanupSummary(repository, policyData, policyName);
      if (request === latestRequest.current) {
        setState({ status: 'done', summary });
      }
    } catch (err: unknown) {
      if (request === latestRequest.current) {
        setState({ status: 'error', error: (err instanceof Error && err.message) || SUMMARY.ERROR });
      }
    }
  };

  const button = (
    <SettingsButton
      variant="secondary"
      icon={Calculator}
      loading={isLoading}
      disabled={!available}
      onClick={calculate}
      data-analytics-id="as-cleanup-policy-summary"
    >
      {SUMMARY.BUTTON}
    </SettingsButton>
  );

  return (
    <Box className="cleanup-policy-summary" aria-live="polite">
      <Flex justify="between" align="start" gap="4" className="cleanup-policy-summary__header">
        <Box>
          <Text as="p" size="3" weight="medium" className="cleanup-policy-summary__title">
            {SUMMARY.TITLE}
          </Text>
          <Text as="p" size="2" color="gray">
            {SUMMARY.DESCRIPTION}
          </Text>
        </Box>
        {available ? (
          button
        ) : (
          <Tooltip content={PREVIEW.BUTTON_TOOLTIP}>
            <Box>{button}</Box>
          </Tooltip>
        )}
      </Flex>

      {isLoading && (
        <Text as="p" size="2" color="gray" mt="3" role="status">
          {SUMMARY.CALCULATING}
        </Text>
      )}

      {state.status === 'error' && (
        <SettingsAlert type="error" className="cleanup-policy-summary__alert">
          {state.error}
        </SettingsAlert>
      )}

      {state.status === 'done' && <SummaryResult summary={state.summary} />}
    </Box>
  );
}

function SummaryResult({ summary }: { summary: CleanupPreviewSummary }) {
  const { repository, componentCount, assetCount, totalSize, complete } = summary;

  if (complete && componentCount === 0) {
    return (
      <SettingsAlert type="info" className="cleanup-policy-summary__alert">
        {SUMMARY.EMPTY(repository)}
      </SettingsAlert>
    );
  }

  return (
    <Box className="cleanup-policy-summary__result">
      <Text as="p" size="2">
        {complete ? SUMMARY.RESULT(repository) : SUMMARY.RESULT_AT_LEAST(repository)}
      </Text>
      <dl className="cleanup-policy-summary__figures">
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
      {!complete && (
        <SettingsAlert type="warning" className="cleanup-policy-summary__alert">
          {SUMMARY.INCOMPLETE}
        </SettingsAlert>
      )}
    </Box>
  );
}

export default CleanupPolicySummary;
