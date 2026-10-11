import React from 'react';
import axios from 'axios';
import {act, render, screen, waitFor} from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import CleanupPolicySummary from './CleanupPolicySummary';
import UIStrings from '../../../constants/UIStrings';

jest.mock('axios', () => ({
  ...jest.requireActual('axios'),
  post: jest.fn(),
}));

const {SUMMARY} = UIStrings.CLEANUP_POLICIES;
const SUMMARY_URL = 'service/rest/internal/cleanup-policies/preview/summary';

const policyData = {
  name: 'zip-files',
  format: 'raw',
  criteriaLastBlobUpdated: '',
  criteriaLastDownloaded: '',
  criteriaReleaseType: '',
  criteriaAssetRegex: '.*\\.zip'
};

const summary = (overrides = {}) => ({
  repository: 'raw-hosted',
  componentCount: 3,
  assetCount: 4,
  totalSize: 2500,
  complete: true,
  ...overrides
});

const calculateButton = () => screen.getByRole('button', {name: SUMMARY.BUTTON});

describe('CleanupPolicySummary', function() {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('does not calculate before a repository is selected', async function() {
    render(<CleanupPolicySummary policyData={policyData} repository=""/>);

    expect(screen.getByText(SUMMARY.TITLE)).toBeInTheDocument();
    expect(calculateButton()).toHaveAttribute('aria-disabled', 'true');

    await act(async () => userEvent.click(calculateButton()));

    expect(axios.post).not.toHaveBeenCalled();
  });

  it('does not calculate without criteria', async function() {
    render(<CleanupPolicySummary policyData={{...policyData, criteriaAssetRegex: ''}} repository="raw-hosted"/>);

    await act(async () => userEvent.click(calculateButton()));

    expect(axios.post).not.toHaveBeenCalled();
  });

  it('shows what the policy would delete once calculated', async function() {
    let resolve;
    axios.post.mockReturnValue(new Promise((r) => {
      resolve = r;
    }));

    render(<CleanupPolicySummary policyData={policyData} repository="raw-hosted"/>);

    await act(async () => userEvent.click(calculateButton()));

    expect(screen.getByText(SUMMARY.CALCULATING)).toBeInTheDocument();
    expect(axios.post).toHaveBeenCalledWith(SUMMARY_URL, {
      repository: 'raw-hosted',
      criteriaLastBlobUpdated: null,
      criteriaLastDownloaded: null,
      criteriaReleaseType: null,
      criteriaAssetRegex: '.*\\.zip'
    });

    await act(async () => resolve({data: summary()}));

    expect(screen.queryByText(SUMMARY.CALCULATING)).not.toBeInTheDocument();
    expect(screen.getByText(/If it ran now, this policy would delete from/)).toBeInTheDocument();
    expect(screen.getByText('raw-hosted')).toBeInTheDocument();
    expect(screen.getByText(SUMMARY.COMPONENTS).nextSibling).toHaveTextContent('3');
    expect(screen.getByText(SUMMARY.FILES).nextSibling).toHaveTextContent('4');
    expect(screen.getByText(SUMMARY.SIZE).nextSibling).toHaveTextContent('2.50 kB');
    expect(screen.queryByText(SUMMARY.INCOMPLETE)).not.toBeInTheDocument();
  });

  it('says "at least" when the server stopped at its time limit', async function() {
    axios.post.mockResolvedValue({data: summary({componentCount: 1200, assetCount: 2400, complete: false})});

    render(<CleanupPolicySummary policyData={policyData} repository="raw-hosted"/>);

    await act(async () => userEvent.click(calculateButton()));

    expect(screen.getByText(/would delete at least the following from/)).toBeInTheDocument();
    expect(screen.getByText(SUMMARY.INCOMPLETE)).toBeInTheDocument();
  });

  it('says so when nothing matches', async function() {
    axios.post.mockResolvedValue({data: summary({componentCount: 0, assetCount: 0, totalSize: 0})});

    render(<CleanupPolicySummary policyData={policyData} repository="raw-hosted"/>);

    await act(async () => userEvent.click(calculateButton()));

    expect(screen.getByText(/Nothing to delete: no component in/)).toBeInTheDocument();
    expect(screen.queryByText(SUMMARY.COMPONENTS)).not.toBeInTheDocument();
  });

  it('shows the server message when the calculation fails', async function() {
    axios.post.mockRejectedValue({
      message: 'Request failed with status code 400',
      response: {
        status: 400,
        data: [{id: '*', message: 'The preview cannot evaluate this policy: Criteria of type(s) [retain] not supported.'}]
      }
    });

    render(<CleanupPolicySummary policyData={{...policyData, retain: 3, sortBy: 'version'}} repository="raw-hosted"/>);

    await act(async () => userEvent.click(calculateButton()));

    expect(axios.post).toHaveBeenCalledWith(SUMMARY_URL, expect.objectContaining({
      criteriaRetain: 3,
      criteriaSortBy: 'version'
    }));
    expect(screen.getByText('The preview cannot evaluate this policy: Criteria of type(s) [retain] not supported.'))
        .toBeInTheDocument();
  });

  it('forgets the totals when the repository changes', async function() {
    axios.post.mockResolvedValue({data: summary()});

    const {rerender} = render(<CleanupPolicySummary policyData={policyData} repository="raw-hosted"/>);

    await act(async () => userEvent.click(calculateButton()));
    expect(screen.getByText(SUMMARY.COMPONENTS)).toBeInTheDocument();

    rerender(<CleanupPolicySummary policyData={policyData} repository="raw-other"/>);

    await waitFor(() => expect(screen.queryByText(SUMMARY.COMPONENTS)).not.toBeInTheDocument());
  });
});
