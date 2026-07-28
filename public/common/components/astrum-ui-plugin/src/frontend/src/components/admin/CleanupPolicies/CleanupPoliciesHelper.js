

import UIStrings from '../../../constants/UIStrings';

const {CLEANUP_POLICIES: LABELS} = UIStrings;

const baseUrl = 'service/rest/internal/cleanup-policies';
const singleCleanupPolicyUrl = (name) => `${baseUrl}/${name}`;

export const isReleaseType = (criteriaReleaseType) =>
  LABELS.RELEASE_TYPE.RELEASES.id === criteriaReleaseType;

export const URL = {
  baseUrl,
  singleCleanupPolicyUrl,
};
