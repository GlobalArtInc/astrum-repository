import React from 'react';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';

import { CopyButton, COPIED_FEEDBACK_MS } from '../CopyButton';
import { copyToClipboard } from '../clipboard';

const mockWriteText = jest.fn();

function renderWithTheme(ui: React.ReactElement) {
  return render(<Theme>{ui}</Theme>);
}

async function click(element: HTMLElement) {
  await act(async () => {
    fireEvent.click(element);
  });
}

describe('CopyButton', () => {
  beforeEach(() => {
    jest.useFakeTimers();
    mockWriteText.mockReset();
    mockWriteText.mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText: mockWriteText }, configurable: true });
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('says Copy until clicked', () => {
    renderWithTheme(<CopyButton text="docker pull x" label="Copy the docker pull snippet" />);
    const button = screen.getByRole('button', { name: 'Copy the docker pull snippet' });
    expect(button).toHaveTextContent('Copy');
    expect(button).toHaveAttribute('data-copied', 'false');
  });

  it('copies the text and says Copied for a moment', async () => {
    const onCopied = jest.fn();
    renderWithTheme(<CopyButton text="docker pull x" onCopied={onCopied} />);

    await click(screen.getByRole('button'));

    expect(mockWriteText).toHaveBeenCalledWith('docker pull x');
    expect(onCopied).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button')).toHaveTextContent('Copied');
    expect(screen.getByRole('button')).toHaveAttribute('data-copied', 'true');

    act(() => {
      jest.advanceTimersByTime(COPIED_FEEDBACK_MS);
    });
    expect(screen.getByRole('button')).toHaveTextContent('Copy');
    expect(screen.getByRole('button')).not.toHaveTextContent('Copied');
  });

  it('forgets the feedback when the text changes', async () => {
    const { rerender } = renderWithTheme(<CopyButton text="one" />);
    await click(screen.getByRole('button'));
    expect(screen.getByRole('button')).toHaveTextContent('Copied');

    rerender(<Theme><CopyButton text="two" /></Theme>);
    expect(screen.getByRole('button')).toHaveTextContent('Copy');
    expect(screen.getByRole('button')).not.toHaveTextContent('Copied');
  });

  it('reports a failed copy and keeps saying Copy', async () => {
    mockWriteText.mockRejectedValue(new Error('denied'));
    const execCommand = jest.fn().mockReturnValue(false);
    Object.defineProperty(document, 'execCommand', { value: execCommand, configurable: true });
    const onCopyFailed = jest.fn();
    renderWithTheme(<CopyButton text="x" onCopyFailed={onCopyFailed} />);

    await click(screen.getByRole('button'));

    expect(onCopyFailed).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button')).toHaveTextContent('Copy');
    expect(screen.getByRole('button')).not.toHaveTextContent('Copied');
  });

  it('renders an icon button with the label as its name', () => {
    renderWithTheme(<CopyButton text="x" iconOnly label="Copy path" />);
    expect(screen.getByRole('button', { name: 'Copy path' })).toBeInTheDocument();
  });

  it('does not let the click reach a clickable row', async () => {
    const onRowClick = jest.fn();
    renderWithTheme(
      <div onClick={onRowClick}>
        <CopyButton text="x" />
      </div>,
    );
    await click(screen.getByRole('button'));
    expect(onRowClick).not.toHaveBeenCalled();
  });
});

describe('copyToClipboard', () => {
  afterEach(() => {
    Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: true });
  });

  it('uses the clipboard API when there is one', async () => {
    const writeText = jest.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    await expect(copyToClipboard('text')).resolves.toBe(true);
    expect(writeText).toHaveBeenCalledWith('text');
  });

  it('falls back to the copy command over plain HTTP, where there is no clipboard API', async () => {
    Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: true });
    let copied = '';
    const execCommand = jest.fn(() => {
      copied = (document.activeElement as HTMLTextAreaElement | null)?.value ?? '';
      return true;
    });
    Object.defineProperty(document, 'execCommand', { value: execCommand, configurable: true });

    await expect(copyToClipboard('pip install x')).resolves.toBe(true);
    expect(execCommand).toHaveBeenCalledWith('copy');
    expect(copied).toBe('pip install x');
    expect(document.querySelector('textarea')).toBeNull();
  });
});
