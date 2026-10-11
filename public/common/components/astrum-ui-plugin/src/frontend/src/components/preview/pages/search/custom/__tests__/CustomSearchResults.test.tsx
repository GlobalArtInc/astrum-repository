import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { CustomSearchResults } from '../CustomSearchResults';

const results = [
  {
    id: 'c1',
    repository: 'npm-hosted',
    format: 'npm',
    group: 'acme',
    name: 'ui-kit',
    version: '2.1.0',
    path: '/@acme/ui-kit/-/ui-kit-2.1.0.tgz',
    lastModified: '2026-10-11T16:55:00Z',
  },
  {
    id: 'c2',
    repository: 'raw-hosted',
    format: 'raw',
    group: '/docs',
    name: '/docs/README.txt',
    version: '',
    path: '/docs/README.txt',
  },
];

function renderResults() {
  render(
    <Theme>
      <CustomSearchResults results={results} loading={false} totalCount={2} isInitial={false} />
    </Theme>
  );
}

describe('CustomSearchResults', () => {
  afterEach(() => {
    window.location.hash = '';
  });

  it('counts the components and names a scoped npm package as npm does', () => {
    renderResults();

    expect(screen.getByText('2 components found')).toBeInTheDocument();
    expect(screen.getByText('@acme/ui-kit')).toBeInTheDocument();
  });

  it('shows a dash rather than "Invalid Date" for a result without a timestamp', () => {
    renderResults();

    expect(screen.queryByText('Invalid Date')).not.toBeInTheDocument();
  });

  it('opens a result at its file in the browse tree of its repository', () => {
    renderResults();

    fireEvent.click(screen.getByRole('button', { name: 'Open @acme/ui-kit in its repository' }));

    expect(window.location.hash).toBe('#preview/browse/npm-hosted/@acme/ui-kit/-/ui-kit-2.1.0.tgz?tab=summary');
  });
});
