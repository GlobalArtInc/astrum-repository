

import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';
import { RepositoryGroupUsageTab } from '../RepositoryGroupUsageTab';
import { useRepositoryTree } from '../useRepositoryTree';

// Mock the hook
jest.mock('../useRepositoryTree');

const renderWithTheme = (component: React.ReactNode) =>
  render(<Theme>{component}</Theme>);

describe('RepositoryGroupUsageTab', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders usage list when repository belongs to groups', () => {
    (useRepositoryTree as jest.Mock).mockReturnValue({
      usages: ['group1', 'group2'],
      loading: false,
      error: undefined,
      refresh: jest.fn(),
    });

    renderWithTheme(<RepositoryGroupUsageTab repositoryName="hosted1" />);

    expect(screen.getByText('group1')).toBeInTheDocument();
    expect(screen.getByText('group2')).toBeInTheDocument();
  });

  it('renders empty message when repository belongs to no groups', () => {
    (useRepositoryTree as jest.Mock).mockReturnValue({
      usages: [],
      loading: false,
      error: undefined,
      refresh: jest.fn(),
    });

    renderWithTheme(<RepositoryGroupUsageTab repositoryName="hosted1" />);

    expect(screen.getByText(/not a member of any groups/i)).toBeInTheDocument();
  });

  it('shows loading state', () => {
    (useRepositoryTree as jest.Mock).mockReturnValue({
      usages: [],
      loading: true,
      error: undefined,
      refresh: jest.fn(),
    });

    renderWithTheme(<RepositoryGroupUsageTab repositoryName="hosted1" />);
    expect(screen.getAllByText(/checking group membership/i)[0]).toBeInTheDocument();
  });
});
