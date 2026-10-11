import React from 'react';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Theme } from '@radix-ui/themes';

import { CleanupPolicySummary } from '../CleanupPolicySummary';
import { CleanupPolicyFormData, CleanupPreviewSummary } from '../types';
import * as useCleanupPoliciesApiModule from '../useCleanupPoliciesApi';
import UIStrings from '../../../../../../../constants/UIStrings';

jest.mock('../useCleanupPoliciesApi');

const mockedUseCleanupPoliciesApi = useCleanupPoliciesApiModule.useCleanupPoliciesApi as jest.MockedFunction<
  typeof useCleanupPoliciesApiModule.useCleanupPoliciesApi
>;

const { SUMMARY } = UIStrings.CLEANUP_POLICIES;

function TestWrapper({ children }: { children: React.ReactNode }) {
  return <Theme>{children}</Theme>;
}

describe('CleanupPolicySummary', () => {
  const mockFetchCleanupSummary = jest.fn();

  const policyData: CleanupPolicyFormData = {
    name: 'zip-files',
    format: 'raw',
    notes: '',
    criteriaLastBlobUpdated: null,
    criteriaLastDownloaded: null,
    criteriaReleaseType: null,
    criteriaAssetRegex: '.*\\.zip',
    retain: null,
    sortBy: null,
  };

  const summary = (overrides: Partial<CleanupPreviewSummary> = {}): CleanupPreviewSummary => ({
    repository: 'raw-hosted',
    componentCount: 3,
    assetCount: 4,
    totalSize: 2500,
    complete: true,
    ...overrides,
  });

  const calculateButton = () => screen.getByRole('button', { name: SUMMARY.BUTTON });

  const renderSummary = (repository = 'raw-hosted', data = policyData) =>
    render(<CleanupPolicySummary policyData={data} repository={repository} policyName="zip-files" />, {
      wrapper: TestWrapper,
    });

  beforeEach(() => {
    jest.clearAllMocks();
    mockedUseCleanupPoliciesApi.mockReturnValue({
      fetchCleanupSummary: mockFetchCleanupSummary,
    } as unknown as ReturnType<typeof useCleanupPoliciesApiModule.useCleanupPoliciesApi>);
  });

  it('cannot calculate before a repository is selected', () => {
    renderSummary('');

    expect(screen.getByText(SUMMARY.TITLE)).toBeInTheDocument();
    expect(calculateButton()).toBeDisabled();
  });

  it('cannot calculate without criteria', () => {
    renderSummary('raw-hosted', { ...policyData, criteriaAssetRegex: null });

    expect(calculateButton()).toBeDisabled();
  });

  it('shows what the policy would delete once calculated', async () => {
    let resolve: (value: CleanupPreviewSummary) => void = () => undefined;
    mockFetchCleanupSummary.mockReturnValue(new Promise((r) => {
      resolve = r;
    }));

    renderSummary();
    await act(async () => userEvent.click(calculateButton()));

    expect(screen.getByText(SUMMARY.CALCULATING)).toBeInTheDocument();
    expect(calculateButton()).toBeDisabled();
    expect(mockFetchCleanupSummary).toHaveBeenCalledWith('raw-hosted', policyData, 'zip-files');

    await act(async () => resolve(summary()));

    expect(screen.queryByText(SUMMARY.CALCULATING)).not.toBeInTheDocument();
    expect(screen.getByText(/If it ran now, this policy would delete from/)).toBeInTheDocument();
    expect(screen.getByText('raw-hosted')).toBeInTheDocument();
    expect(screen.getByText(SUMMARY.COMPONENTS).nextSibling).toHaveTextContent('3');
    expect(screen.getByText(SUMMARY.FILES).nextSibling).toHaveTextContent('4');
    expect(screen.getByText(SUMMARY.SIZE).nextSibling).toHaveTextContent('2.50 kB');
    expect(screen.queryByText(SUMMARY.INCOMPLETE)).not.toBeInTheDocument();
  });

  it('says "at least" when the server stopped at its time limit', async () => {
    mockFetchCleanupSummary.mockResolvedValue(summary({ componentCount: 900, complete: false }));

    renderSummary();
    await act(async () => userEvent.click(calculateButton()));

    expect(screen.getByText(/would delete at least the following from/)).toBeInTheDocument();
    expect(screen.getByText(SUMMARY.COMPONENTS).nextSibling).toHaveTextContent('900');
    expect(screen.getByText(SUMMARY.INCOMPLETE)).toBeInTheDocument();
  });

  it('says so when nothing matches', async () => {
    mockFetchCleanupSummary.mockResolvedValue(summary({ componentCount: 0, assetCount: 0, totalSize: 0 }));

    renderSummary();
    await act(async () => userEvent.click(calculateButton()));

    expect(screen.getByText(/Nothing to delete: no component in/)).toBeInTheDocument();
    expect(screen.queryByText(SUMMARY.COMPONENTS)).not.toBeInTheDocument();
  });

  it('shows the error inline when the calculation fails', async () => {
    mockFetchCleanupSummary.mockRejectedValue(
      new Error('The preview cannot evaluate this policy: Criteria of type(s) [retain] not supported.')
    );

    renderSummary();
    await act(async () => userEvent.click(calculateButton()));

    expect(screen.getByRole('alert')).toHaveTextContent(
      'The preview cannot evaluate this policy: Criteria of type(s) [retain] not supported.'
    );
    expect(calculateButton()).toBeEnabled();
  });

  it('forgets the totals when the repository changes', async () => {
    mockFetchCleanupSummary.mockResolvedValue(summary());

    const { rerender } = renderSummary();
    await act(async () => userEvent.click(calculateButton()));
    expect(screen.getByText(SUMMARY.COMPONENTS)).toBeInTheDocument();

    rerender(<CleanupPolicySummary policyData={policyData} repository="raw-other" policyName="zip-files" />);

    await waitFor(() => expect(screen.queryByText(SUMMARY.COMPONENTS)).not.toBeInTheDocument());
  });
});
