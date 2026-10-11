import React from 'react';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';

import { UsageBlock } from '../UsageBlock';
import type { UsageSnippet } from '../../../pages/search/details/dependencySnippets/usage';

const SNIPPETS: UsageSnippet[] = [
  { id: 'pull', label: 'docker pull', snippetText: 'docker pull host/repo/app:1.0' },
  { id: 'login', label: 'docker login', snippetText: 'docker login host', description: 'Sign in once' },
  { id: 'dockerfile', label: 'Dockerfile', snippetText: 'FROM host/repo/app:1.0' },
];

const mockWriteText = jest.fn();

function renderBlock(props: Partial<React.ComponentProps<typeof UsageBlock>> = {}) {
  return render(
    <Theme>
      <UsageBlock snippets={SNIPPETS} {...props} />
    </Theme>,
  );
}

describe('UsageBlock', () => {
  beforeEach(() => {
    mockWriteText.mockReset();
    mockWriteText.mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText: mockWriteText }, configurable: true });
  });

  it('renders the title, one tab per client and the first snippet', () => {
    renderBlock();
    expect(screen.getByText('Usage')).toBeInTheDocument();
    const tabs = screen.getAllByRole('tab');
    expect(tabs.map((tab) => tab.textContent)).toEqual(['docker pull', 'docker login', 'Dockerfile']);
    expect(tabs[0]).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByRole('tabpanel')).toHaveTextContent('docker pull host/repo/app:1.0');
  });

  it('switches the snippet and its hint with the tab', () => {
    renderBlock();
    expect(screen.queryByText('Sign in once')).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('tab', { name: 'docker login' }));

    expect(screen.getByRole('tab', { name: 'docker login' })).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByRole('tabpanel')).toHaveTextContent('docker login host');
    expect(screen.getByText('Sign in once')).toBeInTheDocument();
  });

  it('moves between tabs with the arrow keys', () => {
    renderBlock();
    const first = screen.getByRole('tab', { name: 'docker pull' });
    fireEvent.keyDown(first, { key: 'ArrowLeft' });
    expect(screen.getByRole('tab', { name: 'Dockerfile' })).toHaveAttribute('aria-selected', 'true');
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Dockerfile' }), { key: 'ArrowRight' });
    expect(first).toHaveAttribute('aria-selected', 'true');
    expect(first).toHaveFocus();
  });

  it('copies the selected snippet', async () => {
    const onCopied = jest.fn();
    renderBlock({ onCopied });
    fireEvent.click(screen.getByRole('tab', { name: 'Dockerfile' }));

    const panel = screen.getByRole('tabpanel');
    await act(async () => {
      fireEvent.click(within(panel).getByRole('button', { name: 'Copy the Dockerfile snippet' }));
    });

    expect(mockWriteText).toHaveBeenCalledWith('FROM host/repo/app:1.0');
    expect(onCopied).toHaveBeenCalledWith(SNIPPETS[2]);
    expect(within(panel).getByRole('button')).toHaveTextContent('Copied');
  });

  it('falls back to the first tab when the snippets change under it', () => {
    const { rerender } = renderBlock();
    fireEvent.click(screen.getByRole('tab', { name: 'Dockerfile' }));
    rerender(
      <Theme>
        <UsageBlock snippets={[{ id: 'npm', label: 'npm', snippetText: 'npm install x' }]} />
      </Theme>,
    );
    expect(screen.getByRole('tab', { name: 'npm' })).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByRole('tabpanel')).toHaveTextContent('npm install x');
  });

  it('renders nothing without snippets', () => {
    renderBlock({ snippets: [] });
    expect(screen.queryByTestId('usage-block')).not.toBeInTheDocument();
  });
});
