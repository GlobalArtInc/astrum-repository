import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from '../api';
import { renderWithApp } from '../test/render';
import { FormatBadge, StatusBadge, TypeBadge } from './Badge';
import { CodeBlock } from './CodeBlock';
import { CopyButton } from './CopyButton';
import { DateTime, RelativeTime } from './DateTime';
import { EmptyState } from './EmptyState';
import { ErrorState } from './ErrorState';
import { FileSize } from './FileSize';
import { Page } from './Page';

afterEach(() => {
  vi.useRealTimers();
});

describe('small widgets', () => {
  it('copy text to the clipboard', async () => {
    const user = userEvent.setup();
    const write = vi.spyOn(navigator.clipboard, 'writeText');
    renderWithApp(<CopyButton text="https://repo/x" label="Copy the URL" />);
    await user.click(screen.getByRole('button', { name: 'Copy the URL' }));
    expect(write).toHaveBeenCalledWith('https://repo/x');
    expect(await navigator.clipboard.readText()).toBe('https://repo/x');
    expect(screen.getAllByText('Copied').length).toBeGreaterThan(0);
  });

  it('show code with a copy button', async () => {
    const user = userEvent.setup();
    renderWithApp(<CodeBlock code="npm install x" title="Shell" />);
    expect(screen.getByText('npm install x').tagName).toBe('CODE');
    await user.click(screen.getByRole('button', { name: 'Copy' }));
    expect(await navigator.clipboard.readText()).toBe('npm install x');
  });

  it('show sizes and dates for the language', () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date('2026-10-12T12:00:00Z'));
    renderWithApp(
      <>
        <FileSize bytes={1536} />
        <RelativeTime value="2026-10-12T10:00:00Z" />
        <DateTime value="2026-10-12T10:00:00Z" dateOnly />
      </>,
      { language: 'ru' },
    );
    expect(screen.getByTitle(/536 байт/)).toHaveTextContent(/^1,5\sКБ$/);
    expect(screen.getByText('2 часа назад')).toHaveAttribute('datetime', '2026-10-12T10:00:00.000Z');
    expect(screen.getByText(/12 окт\. 2026/)).toBeInTheDocument();
  });

  it('name formats, types and states', () => {
    renderWithApp(
      <>
        <FormatBadge format="maven2" />
        <FormatBadge format="unknown-format" />
        <TypeBadge type="proxy" />
        <StatusBadge tone="success">Online</StatusBadge>
      </>,
    );
    expect(screen.getByText('Maven')).toBeInTheDocument();
    expect(screen.getByText('unknown-format')).toBeInTheDocument();
    expect(screen.getByText('Proxy')).toBeInTheDocument();
    expect(screen.getByText('Online')).toBeInTheDocument();
  });

  it('explain empty and failed states', async () => {
    const retry = vi.fn();
    renderWithApp(
      <>
        <EmptyState title="No repositories" description="Create one" />
        <ErrorState error={new ApiError(0, 'Failed to fetch')} onRetry={retry} />
      </>,
    );
    expect(screen.getByRole('heading', { name: 'No repositories' })).toBeInTheDocument();
    expect(screen.getByRole('alert')).toHaveTextContent('The server cannot be reached.');
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(retry).toHaveBeenCalled();
  });

  it('title pages and show breadcrumbs', () => {
    renderWithApp(
      <Page
        title="Users"
        description="People who can sign in"
        breadcrumbs={[{ label: 'Settings', to: '/settings' }, { label: 'Users' }]}
        actions={<button type="button">Create</button>}
      />,
    );
    expect(document.title).toBe('Users - Astrum Repository');
    expect(screen.getByRole('heading', { level: 1, name: 'Users' })).toBeInTheDocument();
    const crumbs = screen.getByRole('navigation', { name: 'Breadcrumbs' });
    expect(crumbs.querySelector('a')).toHaveAttribute('href', '/settings');
    expect(crumbs.querySelector('[aria-current="page"]')).toHaveTextContent('Users');
  });
});
