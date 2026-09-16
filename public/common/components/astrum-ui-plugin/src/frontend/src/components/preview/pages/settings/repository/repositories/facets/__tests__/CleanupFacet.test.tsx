

import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { CleanupFacet } from '../../facets/CleanupFacet';
import { CleanupPolicy, RepositoryFormData } from '../../types';

jest.mock('lucide-react', () => ({
  ExternalLink: () => <span data-testid="external-link-icon" />,
  Search: () => <span data-testid="search-icon" />,
  ChevronRight: () => <span>→</span>,
  ChevronLeft: () => <span>←</span>,
  ChevronsRight: () => <span>⇒</span>,
  ChevronsLeft: () => <span>⇐</span>,
}));

const MAVEN2_FORMAT = 'maven2';
const ALL_FORMATS = '*';
const NPM_FORMAT = 'npm';

const MAVEN_POLICY_NAME = 'maven-only-policy';
const ALL_FORMATS_POLICY_NAME = 'across-all-formats-policy';
const NPM_POLICY_NAME = 'npm-only-policy';

function makePolicy(name: string, format: string): CleanupPolicy {
  return {
    name,
    format,
    notes: '',
    criteriaLastBlobUpdated: null,
    criteriaLastDownloaded: null,
    criteriaReleaseType: null,
    criteriaAssetRegex: null,
    retain: null,
    sortBy: null,
    inUseCount: 0,
  };
}

const defaultFormData: RepositoryFormData = {
  name: 'test-repo',
  type: 'hosted',
  format: MAVEN2_FORMAT,
  online: true,
  storage: {
    blobStoreName: 'default',
    strictContentTypeValidation: true,
    writePolicy: 'ALLOW_ONCE',
  },
  cleanup: null,
};

function renderCleanupFacet(
  props: Partial<Parameters<typeof CleanupFacet>[0]> = {}
) {
  const defaultProps = {
    formData: defaultFormData,
    onChange: jest.fn(),
    onNestedChange: jest.fn(),
    errors: undefined,
    cleanupPolicies: [] as CleanupPolicy[],
  };

  return render(
    <Theme>
      <CleanupFacet {...defaultProps} {...props} />
    </Theme>
  );
}

describe('CleanupFacet', () => {
  describe('available policies filter', () => {
    it('shouldShowFormatMatchingPolicies', () => {
      renderCleanupFacet({
        cleanupPolicies: [makePolicy(MAVEN_POLICY_NAME, MAVEN2_FORMAT)],
      });

      expect(screen.getByText(MAVEN_POLICY_NAME)).toBeInTheDocument();
    });

    it('shouldShowAllFormatsPoliciesAlongsideFormatMatchingPolicies', () => {
      renderCleanupFacet({
        cleanupPolicies: [
          makePolicy(MAVEN_POLICY_NAME, MAVEN2_FORMAT),
          makePolicy(ALL_FORMATS_POLICY_NAME, ALL_FORMATS),
        ],
      });

      expect(screen.getByText(MAVEN_POLICY_NAME)).toBeInTheDocument();
      expect(screen.getByText(ALL_FORMATS_POLICY_NAME)).toBeInTheDocument();
    });

    it('shouldExcludePoliciesForOtherFormats', () => {
      renderCleanupFacet({
        cleanupPolicies: [
          makePolicy(MAVEN_POLICY_NAME, MAVEN2_FORMAT),
          makePolicy(NPM_POLICY_NAME, NPM_FORMAT),
        ],
      });

      expect(screen.getByText(MAVEN_POLICY_NAME)).toBeInTheDocument();
      expect(screen.queryByText(NPM_POLICY_NAME)).not.toBeInTheDocument();
    });

    it('shouldShowAllFormatsPolicyEvenWhenNoFormatMatchExists', () => {
      renderCleanupFacet({
        cleanupPolicies: [makePolicy(ALL_FORMATS_POLICY_NAME, ALL_FORMATS)],
      });

      expect(screen.getByText(ALL_FORMATS_POLICY_NAME)).toBeInTheDocument();
    });
  });
});
