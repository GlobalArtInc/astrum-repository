import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ADMIN_STATE, renderWithApp } from '../../test/render';
import { customersArea } from '../customers';
import { SettingsHubPage } from './SettingsHubPage';

function renderHub(permissions: string[]) {
  return renderWithApp(<SettingsHubPage />, {
    state: { ...ADMIN_STATE, permissions },
    path: '/settings',
    areas: [customersArea],
  });
}

describe('the settings hub', () => {
  it('shows the customers tile to callers who can read customers', async () => {
    renderHub(['astrum:customers:read']);
    expect(await screen.findByRole('link', { name: /Customers/ })).toHaveAttribute(
      'href',
      '/admin/customers',
    );
  });

  it('hides the customers tile from callers without the read privilege', async () => {
    renderHub(['astrum:users:read']);
    expect(await screen.findByText('There is nothing here you can change')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /Customers/ })).not.toBeInTheDocument();
  });
});
