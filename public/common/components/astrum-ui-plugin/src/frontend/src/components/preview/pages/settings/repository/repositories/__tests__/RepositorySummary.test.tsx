import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';

import { RepositorySummary } from '../RepositorySummary';

jest.mock('../BlobStoreStorageCard', () => ({ BlobStoreStorageCard: () => null }));
jest.mock('../RepositoryMetricsCard', () => ({ RepositoryMetricsCard: () => null }));

const FORM_DATA = {
  name: 'docker-onprem',
  type: 'hosted',
  format: 'docker',
  online: false,
  storage: { blobStoreName: 'default', strictContentTypeValidation: true },
};

function renderSummary(repository: Record<string, unknown>) {
  return render(
    <Theme>
      <RepositorySummary
        repository={{ name: 'docker-onprem', type: 'hosted', format: 'docker', url: 'http://localhost/repository/docker-onprem', ...repository } as any}
        formData={FORM_DATA as any}
        onNavigateToTab={jest.fn()}
      />
    </Theme>
  );
}

describe('RepositorySummary', () => {
  it('names the status once when the server gives no description', () => {
    renderSummary({ online: false });

    expect(screen.getAllByText('Offline')).toHaveLength(1);
  });

  it('adds the description of the status the server gives', () => {
    renderSummary({ online: true, status: { online: true, description: 'Ready to Connect' } });

    expect(screen.getAllByText('Online')).toHaveLength(1);
    expect(screen.getByText('Ready to Connect')).toBeInTheDocument();
  });
});
