

import React from 'react';
import { render, screen, } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';

import { RepositorySelector } from '../RepositorySelector';

describe('RepositorySelector', () => {
  const mockRepositories = [
    { name: 'maven-releases', format: 'maven2', url: '' },
    { name: 'npm-hosted', format: 'npm', url: '' },
  ];
  const mockOnChange = jest.fn();

  it('renders the repository selector with options', () => {
    render(
      <Theme>
        <RepositorySelector
          repositoryName="maven-releases"
          onRepositoryChange={mockOnChange}
          availableRepositories={mockRepositories}
        />
      </Theme>
    );

    expect(screen.getByText('Target Repository')).toBeInTheDocument();
    expect(screen.getByDisplayValue('maven-releases')).toBeInTheDocument();
  });
});
