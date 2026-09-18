

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { CalculatedPermissions } from '../CalculatedPermissions';
import { Privilege } from '../../privileges/types';

const mockPrivileges: Privilege[] = [
  {
    id: 'priv1',
    version: '1',
    name: 'Browse Maven',
    description: 'Allows browsing maven repositories',
    type: 'repository-view',
    readOnly: false,
    properties: { format: 'maven2', repository: '*', actions: 'browse' },
    permission: 'astrum:repository-view:maven2:*:browse',
  },
  {
    id: 'priv2',
    version: '1',
    name: 'Read npm',
    description: 'Allows reading npm repositories',
    type: 'repository-view',
    readOnly: false,
    properties: { format: 'npm', repository: '*', actions: 'read' },
    permission: 'astrum:repository-view:npm:*:read',
  }
];

const renderWithTheme = (component: React.ReactNode) => {
  return render(<Theme>{component}</Theme>);
};

describe('CalculatedPermissions', () => {
  it('should render the header', () => {
    renderWithTheme(
      <CalculatedPermissions privileges={mockPrivileges} loading={false} />
    );
    expect(screen.getByText('Calculated Permissions')).toBeInTheDocument();
  });

  it('should render the flattened privileges list', () => {
    renderWithTheme(
      <CalculatedPermissions privileges={mockPrivileges} loading={false} />
    );
    expect(screen.getByText('Browse Maven')).toBeInTheDocument();
    expect(screen.getByText('astrum:repository-view:maven2:*:browse')).toBeInTheDocument();
    expect(screen.getByText('Read npm')).toBeInTheDocument();
  });

  it('should filter privileges by search query', () => {
    renderWithTheme(
      <CalculatedPermissions privileges={mockPrivileges} loading={false} />
    );
    
    const searchInput = screen.getByPlaceholderText('Search permissions...');
    fireEvent.change(searchInput, { target: { value: 'maven' } });

    expect(screen.getByText('Browse Maven')).toBeInTheDocument();
    // EntityTable might not immediately remove non-matching rows in Jest without wait
    expect(screen.queryByText('Read npm')).not.toBeInTheDocument();
  });

  it('should show empty state when no matches found', () => {
    renderWithTheme(
      <CalculatedPermissions privileges={mockPrivileges} loading={false} />
    );
    
    const searchInput = screen.getByPlaceholderText('Search permissions...');
    fireEvent.change(searchInput, { target: { value: 'non-existent' } });

    expect(screen.getByText('No permissions found')).toBeInTheDocument();
  });
});
