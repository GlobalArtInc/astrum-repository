

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { DockerResultRow } from '../DockerResultRow';
import type { DockerResult } from '../docker.types';

const fullResult: DockerResult = {
  id: 'docker:nginx',
  imageName: 'library/nginx',
  displayName: 'nginx',
  latestTag: 'latest',
  tagsCount: 42,
  size: '187 MB',
  lastUpdated: '2024-01-20T10:30:00Z',
  repository: 'docker-hub-proxy',
};

const minimalResult: DockerResult = {
  id: 'docker:minimal',
  imageName: 'local/minimal',
  displayName: 'minimal',
  latestTag: '1.0.0',
  tagsCount: 1,
  lastUpdated: '2024-01-01T00:00:00Z',
};

describe('DockerResultRow', () => {
  it('renders image name and imageName', () => {
    render(
      <table>
        <tbody>
          <DockerResultRow result={fullResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText('nginx')).toBeInTheDocument();
    expect(screen.getByText('library/nginx')).toBeInTheDocument();
  });

  it('renders latestTag and tagsCount', () => {
    render(
      <table>
        <tbody>
          <DockerResultRow result={fullResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText('latest')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
  });

  it('renders size when present', () => {
    render(
      <table>
        <tbody>
          <DockerResultRow result={fullResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText('187 MB')).toBeInTheDocument();
  });

  it('shows dash for size when absent', () => {
    render(
      <table>
        <tbody>
          <DockerResultRow result={minimalResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    const dashes = screen.getAllByText('-');
    expect(dashes.length).toBeGreaterThanOrEqual(1);
  });

  it('renders repository when present', () => {
    render(
      <table>
        <tbody>
          <DockerResultRow result={fullResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    expect(screen.getByText('docker-hub-proxy')).toBeInTheDocument();
  });

  it('shows dash for repository when absent', () => {
    render(
      <table>
        <tbody>
          <DockerResultRow result={minimalResult} onSelect={jest.fn()} />
        </tbody>
      </table>
    );

    const dashes = screen.getAllByText('-');
    expect(dashes.length).toBeGreaterThanOrEqual(1);
  });

  it('calls onSelect when clicked', async () => {
    const onSelect = jest.fn();
    render(
      <table>
        <tbody>
          <DockerResultRow result={fullResult} onSelect={onSelect} />
        </tbody>
      </table>
    );

    const row = screen.getByRole('button', { name: /view details for nginx/i });
    await userEvent.click(row);

    expect(onSelect).toHaveBeenCalledWith('docker:nginx');
  });

  it('calls onSelect when Enter key pressed', () => {
    const onSelect = jest.fn();
    render(
      <table>
        <tbody>
          <DockerResultRow result={fullResult} onSelect={onSelect} />
        </tbody>
      </table>
    );

    const row = screen.getByRole('button', { name: /view details for nginx/i });
    row.focus();
    fireEvent.keyDown(row, { key: 'Enter' });

    expect(onSelect).toHaveBeenCalledWith('docker:nginx');
  });

  it('calls onSelect when Space key pressed', () => {
    const onSelect = jest.fn();
    render(
      <table>
        <tbody>
          <DockerResultRow result={fullResult} onSelect={onSelect} />
        </tbody>
      </table>
    );

    const row = screen.getByRole('button', { name: /view details for nginx/i });
    row.focus();
    fireEvent.keyDown(row, { key: ' ' });

    expect(onSelect).toHaveBeenCalledWith('docker:nginx');
  });
});
