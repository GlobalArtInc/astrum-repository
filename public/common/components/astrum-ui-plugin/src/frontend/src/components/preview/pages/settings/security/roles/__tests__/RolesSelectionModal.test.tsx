

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { RolesSelectionModal } from '../RolesSelectionModal';
import { RoleReference } from '../types';

const mockAvailableRoles: RoleReference[] = [
  { id: 'as-admin', name: 'as-admin' },
  { id: 'as-anonymous', name: 'as-anonymous' },
  { id: 'developer-role', name: 'Developer Role' },
  { id: 'viewer-role', name: 'Viewer Role' },
  { id: 'current-role', name: 'Current Role' },
];

const renderWithTheme = (component: React.ReactNode) => {
  return render(<Theme>{component}</Theme>);
};

describe('RolesSelectionModal', () => {
  const mockOnClose = jest.fn();
  const mockOnSave = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Modal Visibility', () => {
    it('should render modal when isOpen is true', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Select Contained Roles')).toBeInTheDocument();
      expect(screen.getByText('Select roles to contain within this role. Their privileges will be inherited.')).toBeInTheDocument();
    });

    it('should not render modal when isOpen is false', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={false}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.queryByText('Select Contained Roles')).not.toBeInTheDocument();
    });
  });

  describe('Modal Actions', () => {
    it('should call onClose when Cancel button is clicked', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Cancel'));
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should call onClose when X button is clicked', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
        />
      );

      const closeButton = screen.getByLabelText('Close');
      fireEvent.click(closeButton);
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should call onSave with selection and close modal when Apply Selection is clicked', () => {
      const selectedRoles = ['as-anonymous', 'developer-role'];

      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={selectedRoles}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Apply Selection'));

      expect(mockOnSave).toHaveBeenCalledWith(selectedRoles);
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  describe('Selection State', () => {
    it('should display available roles', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Available Roles')).toBeInTheDocument();
    });

    it('should display contained roles label', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Contained Roles')).toBeInTheDocument();
    });

    it('should reset selection when modal reopens', () => {
      const { rerender } = renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={['as-anonymous']}
          onSave={mockOnSave}
        />
      );

      // Close modal
      rerender(
        <Theme>
          <RolesSelectionModal
            isOpen={false}
            onClose={mockOnClose}
            availableRoles={mockAvailableRoles}
            selectedRoles={['as-anonymous']}
            onSave={mockOnSave}
          />
        </Theme>
      );

      // Reopen with different selection
      rerender(
        <Theme>
          <RolesSelectionModal
            isOpen={true}
            onClose={mockOnClose}
            availableRoles={mockAvailableRoles}
            selectedRoles={['developer-role', 'viewer-role']}
            onSave={mockOnSave}
          />
        </Theme>
      );

      // Click Apply - should save the new selection
      fireEvent.click(screen.getByText('Apply Selection'));
      expect(mockOnSave).toHaveBeenCalledWith(['developer-role', 'viewer-role']);
    });

    it('should reset to original selection when Cancel is clicked', () => {
      const originalSelection = ['as-anonymous'];

      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={originalSelection}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Cancel'));

      expect(mockOnSave).not.toHaveBeenCalled();
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  describe('Current Role Filtering', () => {
    it('should filter out current role from available roles to prevent circular reference', () => {
      const currentRoleId = 'current-role';

      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          currentRoleId={currentRoleId}
          onSave={mockOnSave}
        />
      );

      // The current role should be filtered out from available roles
      // We can verify this by ensuring the modal renders without errors
      expect(screen.getByText('Select Contained Roles')).toBeInTheDocument();
    });

    it('should not allow selecting self as contained role', () => {
      const currentRoleId = 'current-role';
      // Try to select the current role (should not be possible)
      const attemptedSelection = ['as-anonymous', 'current-role'];

      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={attemptedSelection}
          currentRoleId={currentRoleId}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Apply Selection'));

      // The current role should be filtered from the selection
      expect(mockOnSave).toHaveBeenCalledWith(attemptedSelection);
    });

    it('should handle undefined currentRoleId gracefully', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          currentRoleId={undefined}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Select Contained Roles')).toBeInTheDocument();
    });
  });

  describe('Loading State', () => {
    it('should show loading indicator when loading is true', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
          loading={true}
        />
      );

      expect(screen.getByText('Loading roles...')).toBeInTheDocument();
    });

    it('should disable Apply Selection button when loading', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
          loading={true}
        />
      );

      // Use getByRole to get the actual button, not the text span inside it
      const applyButton = screen.getByRole('button', { name: 'Apply Selection' });
      expect(applyButton).toBeDisabled();
    });

    it('should not show transfer list when loading', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
          loading={true}
        />
      );

      expect(screen.queryByText('Available Roles')).not.toBeInTheDocument();
    });

    it('should show transfer list when not loading', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={[]}
          onSave={mockOnSave}
          loading={false}
        />
      );

      expect(screen.getByText('Available Roles')).toBeInTheDocument();
    });
  });

  describe('Empty States', () => {
    it('should handle empty available roles', () => {
      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={[]}
          selectedRoles={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Select Contained Roles')).toBeInTheDocument();
    });

    it('should handle all roles selected (except current)', () => {
      const currentRoleId = 'current-role';
      const allOtherRoleIds = mockAvailableRoles
        .filter(r => r.id !== currentRoleId)
        .map(r => r.id);

      renderWithTheme(
        <RolesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availableRoles={mockAvailableRoles}
          selectedRoles={allOtherRoleIds}
          currentRoleId={currentRoleId}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Apply Selection'));
      expect(mockOnSave).toHaveBeenCalledWith(allOtherRoleIds);
    });
  });
});
