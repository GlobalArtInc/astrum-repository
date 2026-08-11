

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import PlaceholderPage from '../PlaceholderPage';

// Mock window.location
const mockLocation = {
  hash: '',
};
Object.defineProperty(window, 'location', {
  value: mockLocation,
  writable: true,
});

describe('PlaceholderPage', () => {
  const defaultProps = {
    featureName: 'Test Feature',
    defaultUrl: '#admin/repository/repositories',
  };

  beforeEach(() => {
    jest.clearAllMocks();
    mockLocation.hash = '';
  });

  describe('rendering', () => {
    it('renders feature name', () => {
      render(<PlaceholderPage {...defaultProps} />);
      expect(screen.getByText('Test Feature')).toBeInTheDocument();
    });

    it('renders PREVIEW label', () => {
      render(<PlaceholderPage {...defaultProps} />);
      expect(screen.getByText('PREVIEW:')).toBeInTheDocument();
    });

    it('renders TODO status', () => {
      render(<PlaceholderPage {...defaultProps} />);
      expect(screen.getByText('TODO')).toBeInTheDocument();
    });

    it('renders Use Current Version button', () => {
      render(<PlaceholderPage {...defaultProps} />);
      expect(screen.getByRole('button', { name: /Use Current Version/i })).toBeInTheDocument();
    });
  });

  describe('navigation', () => {
    it('navigates to default URL when button is clicked', () => {
      render(<PlaceholderPage {...defaultProps} />);
      fireEvent.click(screen.getByRole('button', { name: /Use Current Version/i }));
      expect(mockLocation.hash).toBe('#admin/repository/repositories');
    });

    it('handles different default URLs', () => {
      render(<PlaceholderPage featureName="Security" defaultUrl="#admin/security/users" />);
      fireEvent.click(screen.getByRole('button', { name: /Use Current Version/i }));
      expect(mockLocation.hash).toBe('#admin/security/users');
    });
  });
});


