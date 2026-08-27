

import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { useCurrentStateAndParams } from '@uirouter/react';

import { UploadPage } from '../UploadPage';

// Mock the router hook
jest.mock('@uirouter/react', () => ({
  useCurrentStateAndParams: jest.fn(),
}));

// Mock sub-components
jest.mock('../UploadRepositoryListPage', () => ({
  UploadRepositoryListPage: () => <div data-testid="upload-list-page">List Page</div>,
}));
jest.mock('../UploadFormContainer', () => ({
  UploadFormContainer: () => <div data-testid="upload-form-container">Form Page</div>,
}));

describe('UploadPage Router', () => {
  const mockUseCurrentStateAndParams = useCurrentStateAndParams as jest.Mock;

  it('renders UploadRepositoryListPage when no repoName in params', () => {
    mockUseCurrentStateAndParams.mockReturnValue({ params: {} });
    render(<Theme><UploadPage /></Theme>);
    expect(screen.getByTestId('upload-list-page')).toBeInTheDocument();
  });

  it('renders UploadFormContainer when repoName is present in params', () => {
    mockUseCurrentStateAndParams.mockReturnValue({ params: { repoName: 'maven-releases' } });
    render(<Theme><UploadPage /></Theme>);
    expect(screen.getByTestId('upload-form-container')).toBeInTheDocument();
  });
});
