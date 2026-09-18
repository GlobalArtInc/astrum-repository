

import React from 'react';
import { render, screen, fireEvent, } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { PrivilegesSelectionModal } from '../PrivilegesSelectionModal';
import { PrivilegeReference } from '../types';

const mockAvailablePrivileges: PrivilegeReference[] = [
  { id: 'as-all', name: 'as-all' },
  { id: 'as-search-read', name: 'as-search-read' },
  { id: 'as-repository-view-*', name: 'as-repository-view-*' },
  { id: 'as-admin', name: 'as-admin' },
  { id: 'as-repository-admin', name: 'as-repository-admin' },
];

const renderWithTheme = (component: React.ReactNode) => {
  return render(<Theme>{component}</Theme>);
};

describe('PrivilegesSelectionModal', () => {
  const mockOnClose = jest.fn();
  const mockOnSave = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Modal Visibility', () => {
    it('should render modal when isOpen is true', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Select Privileges')).toBeInTheDocument();
      expect(screen.getByText('Select privileges to grant to this role. Double-click or use arrows to move items.')).toBeInTheDocument();
    });

    it('should not render modal when isOpen is false', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={false}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.queryByText('Select Privileges')).not.toBeInTheDocument();
    });
  });

  describe('Modal Actions', () => {
    it('should call onClose when Cancel button is clicked', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Cancel'));
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should call onClose when X button is clicked', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
        />
      );

      const closeButton = screen.getByLabelText('Close');
      fireEvent.click(closeButton);
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should call onSave with selection and close modal when Apply Selection is clicked', () => {
      const selectedPrivileges = ['as-search-read', 'as-admin'];

      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={selectedPrivileges}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Apply Selection'));

      expect(mockOnSave).toHaveBeenCalledWith(selectedPrivileges);
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  describe('Selection State', () => {
    it('should display available privileges', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Available Privileges')).toBeInTheDocument();
    });

    it('should display selected privileges label', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Selected Privileges')).toBeInTheDocument();
    });

    it('should reset selection when modal reopens', async () => {
      const { rerender } = renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={['as-search-read']}
          onSave={mockOnSave}
        />
      );

      // Close modal
      rerender(
        <Theme>
          <PrivilegesSelectionModal
            isOpen={false}
            onClose={mockOnClose}
            availablePrivileges={mockAvailablePrivileges}
            selectedPrivileges={['as-search-read']}
            onSave={mockOnSave}
          />
        </Theme>
      );

      // Reopen with different selection
      rerender(
        <Theme>
          <PrivilegesSelectionModal
            isOpen={true}
            onClose={mockOnClose}
            availablePrivileges={mockAvailablePrivileges}
            selectedPrivileges={['as-admin', 'as-all']}
            onSave={mockOnSave}
          />
        </Theme>
      );

      // Click Apply - should save the new selection
      fireEvent.click(screen.getByText('Apply Selection'));
      expect(mockOnSave).toHaveBeenCalledWith(['as-admin', 'as-all']);
    });

    it('should reset to original selection when Cancel is clicked', () => {
      const originalSelection = ['as-search-read'];

      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={originalSelection}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Cancel'));

      expect(mockOnSave).not.toHaveBeenCalled();
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  describe('Loading State', () => {
    it('should show loading indicator when loading is true', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
          loading={true}
        />
      );

      expect(screen.getByText('Loading privileges...')).toBeInTheDocument();
    });

    it('should disable Apply Selection button when loading', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
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
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
          loading={true}
        />
      );

      expect(screen.queryByText('Available Privileges')).not.toBeInTheDocument();
    });

    it('should show transfer list when not loading', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={[]}
          onSave={mockOnSave}
          loading={false}
        />
      );

      expect(screen.getByText('Available Privileges')).toBeInTheDocument();
    });
  });

  describe('Empty States', () => {
    it('should handle empty available privileges', () => {
      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={[]}
          selectedPrivileges={[]}
          onSave={mockOnSave}
        />
      );

      expect(screen.getByText('Select Privileges')).toBeInTheDocument();
    });

    it('should handle all privileges selected', () => {
      const allPrivilegeIds = mockAvailablePrivileges.map(p => p.id);

      renderWithTheme(
        <PrivilegesSelectionModal
          isOpen={true}
          onClose={mockOnClose}
          availablePrivileges={mockAvailablePrivileges}
          selectedPrivileges={allPrivilegeIds}
          onSave={mockOnSave}
        />
      );

      fireEvent.click(screen.getByText('Apply Selection'));
      expect(mockOnSave).toHaveBeenCalledWith(allPrivilegeIds);
    });
  });
});
