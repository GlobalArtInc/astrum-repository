import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { RepositoryTab } from '../tabs/RepositoryTab';
import type { RepositoryProfileData } from '../types';

const repository = {
  name: 'raw-docs',
  format: 'raw',
  type: 'hosted',
  url: 'http://localhost:8081/repository/raw-docs/',
  online: true,
  attributes: {
    storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    cleanup: { policyNames: ['demo-cleanup', 'old-snapshots'] },
  },
} as RepositoryProfileData;

describe('RepositoryTab', () => {
  it('lists the cleanup policies the server names in cleanup.policyNames', () => {
    render(
      <Theme>
        <RepositoryTab repository={repository} blobStore={null} cleanupPolicies={[]} routingRule={null} />
      </Theme>
    );

    expect(screen.getByText('demo-cleanup')).toBeInTheDocument();
    expect(screen.getByText('old-snapshots')).toBeInTheDocument();
  });
});
