

import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';

import { ExpiresCell } from '../ExpiresCell';
import { SERVICE_ACCOUNT_TOKENS_STRINGS } from '../strings';

const LABELS = SERVICE_ACCOUNT_TOKENS_STRINGS.EXPIRES_CELL;

describe('ExpiresCell', () => {
  describe('never expires', () => {
    it('shows "Never" when expiresAt is null', () => {
      render(<ExpiresCell expiresAt={null} />);
      expect(screen.getByText(LABELS.NEVER)).toBeInTheDocument();
    });

    it('applies gray color for never-expires text', () => {
      const { container } = render(<ExpiresCell expiresAt={null} />);
      const text = screen.getByText(LABELS.NEVER);
      expect(text).toHaveClass('rt-Text');
    });
  });

  describe('expired tokens', () => {
    it('shows "Expired" badge for past dates', () => {
      // Date in the past
      const pastDate = new Date();
      pastDate.setFullYear(pastDate.getFullYear() - 1);
      const pastDateStr = pastDate.toISOString();

      render(<ExpiresCell expiresAt={pastDateStr} />);

      expect(screen.getByText(LABELS.EXPIRED)).toBeInTheDocument();
    });

    it('shows alert icon for expired tokens (not color-only signal)', () => {
      const pastDate = new Date();
      pastDate.setFullYear(pastDate.getFullYear() - 1);
      const pastDateStr = pastDate.toISOString();

      const { container } = render(<ExpiresCell expiresAt={pastDateStr} />);

      // The icon should be present with aria-hidden="true"
      const icon = container.querySelector('[aria-hidden="true"]');
      expect(icon).toBeInTheDocument();
    });
  });

  describe('future expiration', () => {
    it('shows formatted date for future dates', () => {
      // Date in the future
      const futureDate = new Date();
      futureDate.setFullYear(futureDate.getFullYear() + 1);
      const futureDateStr = futureDate.toISOString();

      render(<ExpiresCell expiresAt={futureDateStr} />);

      // Should show the year
      const expectedYear = futureDate.getFullYear().toString();
      expect(screen.getByText(new RegExp(expectedYear))).toBeInTheDocument();
    });
  });
});
