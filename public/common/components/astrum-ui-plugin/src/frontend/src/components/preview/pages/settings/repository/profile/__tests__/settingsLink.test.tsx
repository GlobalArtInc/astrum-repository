import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { SettingsLink } from '../tabs/settingsLink';
import { blobStorePath } from '../tabs/RepositoryTab';

function renderWithTheme(ui: React.ReactElement) {
  return render(<Theme>{ui}</Theme>);
}

describe('SettingsLink', () => {
  it('links to the Astrum One page in the same tab with the default "Configure" label', () => {
    renderWithTheme(<SettingsLink previewPath="admin/repository/repositories/npm-proxy" />);

    const link = screen.getByRole('link', { name: 'Configure' });
    expect(link).toHaveAttribute('href', '#admin/repository/repositories/npm-proxy');
    expect(link).not.toHaveAttribute('target');
  });

  it('renders a custom label', () => {
    renderWithTheme(<SettingsLink previewPath="admin/security/privileges" label="View" />);

    expect(screen.getByRole('link', { name: 'View' })).toHaveAttribute('href', '#admin/security/privileges');
  });
});

describe('blobStorePath', () => {
  it('opens the edit page of the blob store when its type is known', () => {
    expect(blobStorePath('my store', { name: 'my store', type: 'File', typeId: 'file', unavailable: false }))
      .toBe('admin/repository/blobstores/file/my%20store');
  });

  it('opens the blob store list when the type is not known', () => {
    expect(blobStorePath('default', null)).toBe('admin/repository/blobstores');
  });

  it('has no link without a blob store', () => {
    expect(blobStorePath(undefined, null)).toBeUndefined();
  });
});
