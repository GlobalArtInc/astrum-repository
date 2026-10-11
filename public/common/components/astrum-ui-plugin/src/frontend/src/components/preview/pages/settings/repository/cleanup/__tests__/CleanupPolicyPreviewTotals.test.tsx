import React from 'react';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Theme } from '@radix-ui/themes';

import { CleanupPolicyDryRun } from '../CleanupPolicyDryRun';
import { CleanupPolicyPreview } from '../CleanupPolicyPreview';
import { CleanupPolicyFormData } from '../types';
import * as useCleanupPoliciesApiModule from '../useCleanupPoliciesApi';
import UIStrings from '../../../../../../../constants/UIStrings';

jest.mock('../useCleanupPoliciesApi');

// Radix Select cannot be driven in jsdom; a native select exposes the same contract.
jest.mock('../../../../../shared/form', () => ({
  ...jest.requireActual('../../../../../shared/form'),
  SettingsSelect: ({
    name,
    value,
    onChange,
    options,
  }: {
    name: string;
    value: string;
    onChange: (value: string) => void;
    options: { value: string; label: string }[];
  }) => (
    <select aria-label={name} value={value} onChange={(e) => onChange(e.target.value)}>
      <option value="">-</option>
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  ),
}));

const mockedUseCleanupPoliciesApi = useCleanupPoliciesApiModule.useCleanupPoliciesApi as jest.MockedFunction<
  typeof useCleanupPoliciesApiModule.useCleanupPoliciesApi
>;

const { DRY_RUN, SUMMARY } = UIStrings.CLEANUP_POLICIES;

function TestWrapper({ children }: { children: React.ReactNode }) {
  return <Theme>{children}</Theme>;
}

describe('cleanup policy totals', () => {
  const mockFetchRepositories = jest.fn();
  const mockPreviewCleanupPolicy = jest.fn();
  const mockFetchCleanupSummary = jest.fn();

  const policyData: CleanupPolicyFormData = {
    name: 'old-releases',
    format: 'maven2',
    notes: '',
    criteriaLastBlobUpdated: 30,
    criteriaLastDownloaded: null,
    criteriaReleaseType: null,
    criteriaAssetRegex: null,
    retain: null,
    sortBy: null,
  };

  beforeEach(() => {
    jest.clearAllMocks();
    mockFetchRepositories.mockResolvedValue([{ id: 'maven-releases', name: 'maven-releases' }]);
    mockFetchCleanupSummary.mockResolvedValue({
      repository: 'maven-releases',
      componentCount: 120,
      assetCount: 480,
      totalSize: 5_000_000_000,
      complete: true,
    });
    mockedUseCleanupPoliciesApi.mockReturnValue({
      fetchRepositories: mockFetchRepositories,
      previewCleanupPolicy: mockPreviewCleanupPolicy,
      fetchCleanupSummary: mockFetchCleanupSummary,
      getDryRunCsvUrl: jest.fn().mockReturnValue('/csv'),
    } as unknown as ReturnType<typeof useCleanupPoliciesApiModule.useCleanupPoliciesApi>);
  });

  async function selectRepository(name: string) {
    await waitFor(() => expect(screen.getByRole('option', { name: 'maven-releases' })).toBeInTheDocument());
    await act(async () => userEvent.selectOptions(screen.getByLabelText(name), 'maven-releases'));
  }

  describe('CleanupPolicyDryRun', () => {
    it('calls the sample a sample instead of showing the unknown total', async () => {
      mockPreviewCleanupPolicy.mockResolvedValue({
        components: [
          { name: 'app', group: 'org.example', version: '1.0', repository: 'maven-releases' },
          { name: 'lib', group: 'org.example', version: '2.0', repository: 'maven-releases' },
        ],
        total: -1,
      });

      render(<CleanupPolicyDryRun policyData={policyData} policyName="old-releases" />, { wrapper: TestWrapper });
      await selectRepository('dryRunRepository');

      await waitFor(() => expect(screen.getByText(DRY_RUN.SAMPLE_COUNT(2))).toBeInTheDocument());
      expect(screen.queryByText(/of -1/)).not.toBeInTheDocument();
      expect(screen.getByText('app')).toBeInTheDocument();
    });

    it('shows a single empty state when nothing matches', async () => {
      mockPreviewCleanupPolicy.mockResolvedValue({ components: [], total: -1 });

      render(<CleanupPolicyDryRun policyData={policyData} policyName="old-releases" />, { wrapper: TestWrapper });
      await selectRepository('dryRunRepository');

      await waitFor(() =>
        expect(screen.getByText('No components match the selected criteria')).toBeInTheDocument()
      );
      expect(screen.queryByText(DRY_RUN.SAMPLE_COUNT(0))).not.toBeInTheDocument();
      expect(screen.queryByText('No assets in this repository match the current criteria.')).not.toBeInTheDocument();
    });

    it('calculates the totals for the selected repository', async () => {
      mockPreviewCleanupPolicy.mockResolvedValue({ components: [], total: -1 });

      render(<CleanupPolicyDryRun policyData={policyData} policyName="old-releases" />, { wrapper: TestWrapper });
      await selectRepository('dryRunRepository');

      await act(async () => userEvent.click(screen.getByRole('button', { name: SUMMARY.BUTTON })));

      expect(mockFetchCleanupSummary).toHaveBeenCalledWith('maven-releases', policyData, 'old-releases');
      expect(screen.getByText(SUMMARY.COMPONENTS).nextSibling).toHaveTextContent('120');
      expect(screen.getByText(SUMMARY.FILES).nextSibling).toHaveTextContent('480');
      expect(screen.getByText(SUMMARY.SIZE).nextSibling).toHaveTextContent('5.00 GB');
    });
  });

  describe('CleanupPolicyPreview', () => {
    it('calculates the totals for the selected repository', async () => {
      render(<CleanupPolicyPreview policyData={policyData} />, { wrapper: TestWrapper });
      await selectRepository('previewRepository');

      await act(async () => userEvent.click(screen.getByRole('button', { name: SUMMARY.BUTTON })));

      expect(mockFetchCleanupSummary).toHaveBeenCalledWith('maven-releases', policyData, undefined);
      expect(screen.getByText(SUMMARY.COMPONENTS).nextSibling).toHaveTextContent('120');
      expect(mockPreviewCleanupPolicy).not.toHaveBeenCalled();
    });
  });
});
