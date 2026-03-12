import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { ApiError } from '../api';
import { renderWithApp } from '../test/render';
import { ConfirmDialog, useConfirm } from './ConfirmDialog';

function Asker({ onAnswer }: { onAnswer(answer: boolean): void }) {
  const confirm = useConfirm();
  return (
    <button
      type="button"
      onClick={async () =>
        onAnswer(await confirm({ title: 'Delete raw-files?', confirmLabel: 'Delete', danger: true }))
      }
    >
      ask
    </button>
  );
}

describe('confirmations', () => {
  it('answer yes or no', async () => {
    const answer = vi.fn();
    renderWithApp(<Asker onAnswer={answer} />);
    await userEvent.click(screen.getByRole('button', { name: 'ask' }));
    expect(await screen.findByRole('alertdialog', { name: 'Delete raw-files?' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
    expect(answer).toHaveBeenLastCalledWith(true);
    await userEvent.click(screen.getByRole('button', { name: 'ask' }));
    await userEvent.click(await screen.findByRole('button', { name: 'Cancel' }));
    expect(answer).toHaveBeenLastCalledWith(false);
  });

  it('stay open and show why an action failed', async () => {
    function Failing() {
      const [open, setOpen] = useState(true);
      return (
        <ConfirmDialog
          open={open}
          onOpenChange={setOpen}
          title="Delete it?"
          onConfirm={() => Promise.reject(new ApiError(400, 'the blob store is in use'))}
        />
      );
    }
    renderWithApp(<Failing />);
    await userEvent.click(await screen.findByRole('button', { name: 'Confirm' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('the blob store is in use');
    expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  });
});
