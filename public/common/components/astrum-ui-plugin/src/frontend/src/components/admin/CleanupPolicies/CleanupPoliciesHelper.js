

import UIStrings from '../../../constants/UIStrings';

const {CLEANUP_POLICIES: LABELS} = UIStrings;

const baseUrl = 'service/rest/internal/cleanup-policies';
const singleCleanupPolicyUrl = (name) => `${baseUrl}/${name}`;
const previewSummaryUrl = `${baseUrl}/preview/summary`;

export const isReleaseType = (criteriaReleaseType) =>
  LABELS.RELEASE_TYPE.RELEASES.id === criteriaReleaseType;

export const URL = {
  baseUrl,
  singleCleanupPolicyUrl,
  previewSummaryUrl,
};

/**
 * Body of the preview requests for the policy being edited.
 */
export function previewRequest(repository, policyData) {
  const request = {
    repository,
    criteriaLastBlobUpdated: policyData.criteriaLastBlobUpdated || null,
    criteriaLastDownloaded: policyData.criteriaLastDownloaded || null,
    criteriaReleaseType: policyData.criteriaReleaseType || null,
    criteriaAssetRegex: policyData.criteriaAssetRegex || null
  };
  if (policyData.retain) {
    request.criteriaRetain = policyData.retain;
  }
  if (policyData.sortBy) {
    request.criteriaSortBy = policyData.sortBy;
  }
  return request;
}

/**
 * The message of a failed preview request: the validation errors of a 400, the text the server sent, or the
 * request's own message.
 */
export function previewErrorMessage(error) {
  const data = error?.response?.data;
  if (Array.isArray(data)) {
    const messages = data.map(({message}) => message).filter(Boolean);
    if (messages.length) {
      return messages.join(' ');
    }
  }
  if (typeof data === 'string' && data.trim()) {
    return data;
  }
  if (data?.message) {
    return data.message;
  }
  return error?.message;
}
