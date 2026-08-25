

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { GAResultRow } from '../GAResultRow';
import type { GAResult } from '../../core';

jest.mock('../../../../shared', () => ({
  FormatBadge: ({ format }: { format: string }) => <span data-testid="format-badge">{format}</span>,
}));

const fullResult: GAResult = {
  gaId: 'maven:org.apache.commons:commons-lang3',
  format: 'maven2' as any,
  displayName: 'commons-lang3',
  namespace: 'org.apache.commons',
  latestVersion: '3.14.0',
  versionsCount: 42,
  repositoriesCount: 5,
  lastUpdated: '2024-01-20T10:30:00Z',
};

const noVersionResult: GAResult = {
  gaId: 'maven:org.example:lib',
  format: 'maven2' as any,
  displayName: 'lib',
  namespace: 'org.example',
  versionsCount: 1,
  repositoriesCount: 1,
  lastUpdated: '2024-01-01T00:00:00Z',
};

describe('GAResultRow', () => {
  it('renders display name', () => {
    render(<GAResultRow result={fullResult} onSelect={jest.fn()} />);

    expect(screen.getByText('commons-lang3')).toBeInTheDocument();
  });

  it('renders namespace', () => {
    render(<GAResultRow result={fullResult} onSelect={jest.fn()} />);

    expect(screen.getByText('org.apache.commons')).toBeInTheDocument();
  });

  it('renders latestVersion when present', () => {
    render(<GAResultRow result={fullResult} onSelect={jest.fn()} />);

    expect(screen.getByText(/3\.14\.0/)).toBeInTheDocument();
  });

  it('shows dash for latestVersion when absent', () => {
    render(<GAResultRow result={noVersionResult} onSelect={jest.fn()} />);

    expect(screen.getByText(/Latest: -/)).toBeInTheDocument();
  });

  it('renders format badge', () => {
    render(<GAResultRow result={fullResult} onSelect={jest.fn()} />);

    const badges = screen.getAllByTestId('format-badge');
    expect(badges.length).toBeGreaterThanOrEqual(1);
    expect(badges[0]).toHaveTextContent('maven2');
  });

  it('calls onSelect when clicked', async () => {
    const onSelect = jest.fn();
    render(<GAResultRow result={fullResult} onSelect={onSelect} />);

    const row = screen.getByRole('button', { name: /view details for commons-lang3/i });
    await userEvent.click(row);

    expect(onSelect).toHaveBeenCalledWith('maven:org.apache.commons:commons-lang3');
  });

  it('calls onSelect when Enter key pressed', () => {
    const onSelect = jest.fn();
    render(<GAResultRow result={fullResult} onSelect={onSelect} />);

    const row = screen.getByRole('button', { name: /view details for commons-lang3/i });
    row.focus();
    fireEvent.keyDown(row, { key: 'Enter' });

    expect(onSelect).toHaveBeenCalledWith('maven:org.apache.commons:commons-lang3');
  });

  it('calls onSelect when Space key pressed', () => {
    const onSelect = jest.fn();
    render(<GAResultRow result={fullResult} onSelect={onSelect} />);

    const row = screen.getByRole('button', { name: /view details for commons-lang3/i });
    row.focus();
    fireEvent.keyDown(row, { key: ' ' });

    expect(onSelect).toHaveBeenCalledWith('maven:org.apache.commons:commons-lang3');
  });
});
