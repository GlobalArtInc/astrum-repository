import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { renderWithApp } from '../../../test/render';
import { UsageSnippets } from './UsageSnippets';

const IMAGE = 'repo.example.org/docker-hosted/team/app:1.2';

describe('container usage', () => {
  it('gives each client a titled snippet to pull, log in and push', async () => {
    renderWithApp(
      <UsageSnippets
        context={{
          format: 'docker',
          repositoryName: 'docker-hosted',
          repositoryType: 'hosted',
          baseUrl: 'https://repo.example.org',
          component: { group: '', name: 'team/app', version: '1.2' },
        }}
      />,
    );
    expect(screen.getAllByRole('tab').map((tab) => tab.textContent)).toEqual([
      'Docker',
      'Podman',
      'Dockerfile',
      'Compose',
      'Kubernetes',
    ]);
    expect(
      within(screen.getByRole('region', { name: 'Pull' })).getByText(`docker pull ${IMAGE}`),
    ).toBeInTheDocument();
    expect(
      within(screen.getByRole('region', { name: 'Login' })).getByText('docker login repo.example.org'),
    ).toBeInTheDocument();
    expect(within(screen.getByRole('region', { name: 'Push' })).getByText(/docker push/)).toBeInTheDocument();
    expect(screen.getAllByRole('button', { name: /^Copy the Docker/ })).toHaveLength(3);

    await userEvent.click(screen.getByRole('tab', { name: 'Podman' }));
    expect(
      within(screen.getByRole('region', { name: 'Pull' })).getByText(`podman pull ${IMAGE}`),
    ).toBeInTheDocument();

    await userEvent.click(screen.getByRole('tab', { name: 'Compose' }));
    expect(screen.queryByRole('region', { name: 'Pull' })).not.toBeInTheDocument();
  });
});
