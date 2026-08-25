

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { NpmResultRow } from '../NpmResultRow';
import type { NpmResult } from '../npm.types';

const fullResult: NpmResult = {
  id: 'npm:lodash',
  scope: '',
  name: 'lodash',
  displayName: 'lodash',
  latestVersion: '4.17.21',
  versionsCount: 89,
  description: 'Lodash modular utilities',
  author: 'John-David Dalton',
  license: 'MIT',
  repositoriesCount: 3,
  lastUpdated: '2024-01-15T12:00:00Z',
};

const minimalResult: NpmResult = {
  id: 'npm:minimal',
  scope: '',
  name: 'minimal',
  displayName: 'minimal',
  latestVersion: '1.0.0',
  versionsCount: 1,
  repositoriesCount: 1,
  lastUpdated: '2024-01-01T00:00:00Z',
};

describe('NpmResultRow', () => {
  it('renders display name', () => {
    render(
      <table>
        <tbody>
          <NpmResultRow result={fullResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText('lodash')).toBeInTheDocument();
  });

  it('renders description when present', () => {
    render(
      <table>
        <tbody>
          <NpmResultRow result={fullResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText(/Lodash modular utilities/)).toBeInTheDocument();
  });

  it('does not render description when absent', () => {
    render(
      <table>
        <tbody>
          <NpmResultRow result={minimalResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.queryByText(/utilities/)).not.toBeInTheDocument();
  });

  it('renders author when present', () => {
    render(
      <table>
        <tbody>
          <NpmResultRow result={fullResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText('John-David Dalton')).toBeInTheDocument();
  });

  it('shows dash for author when absent', () => {
    render(
      <table>
        <tbody>
          <NpmResultRow result={minimalResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText('-')).toBeInTheDocument();
  });

  it('renders latestVersion and versionsCount', () => {
    render(
      <table>
        <tbody>
          <NpmResultRow result={fullResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText('4.17.21')).toBeInTheDocument();
    expect(screen.getByText('89')).toBeInTheDocument();
  });

  it('calls onSelect when clicked', async () => {
    const onSelect = jest.fn();
    render(
      <table>
        <tbody>
          <NpmResultRow result={fullResult} onSelect={onSelect} />
        </tbody>
      </table>
    );

    const row = screen.getByRole('button', { name: /view details for lodash/i });
    await userEvent.click(row);

    expect(onSelect).toHaveBeenCalledWith('npm:lodash');
  });

  it('calls onSelect when Enter key pressed', () => {
    const onSelect = jest.fn();
    render(
      <table>
        <tbody>
          <NpmResultRow result={fullResult} onSelect={onSelect} />
        </tbody>
      </table>
    );

    const row = screen.getByRole('button', { name: /view details for lodash/i });
    row.focus();
    fireEvent.keyDown(row, { key: 'Enter' });

    expect(onSelect).toHaveBeenCalledWith('npm:lodash');
  });

  it('calls onSelect when Space key pressed', () => {
    const onSelect = jest.fn();
    render(
      <table>
        <tbody>
          <NpmResultRow result={fullResult} onSelect={onSelect} />
        </tbody>
      </table>
    );

    const row = screen.getByRole('button', { name: /view details for lodash/i });
    row.focus();
    fireEvent.keyDown(row, { key: ' ' });

    expect(onSelect).toHaveBeenCalledWith('npm:lodash');
  });
});
