import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { ConfirmDialog } from '../ConfirmDialog';

describe('ConfirmDialog', () => {
  function renderDialog(onConfirm = jest.fn()) {
    render(
      <Theme appearance="dark">
        <ConfirmDialog
          open
          onOpenChange={jest.fn()}
          title="Revoke token"
          message="Clients that use it stop working."
          confirmLabel="Revoke"
          onConfirm={onConfirm}
        />
      </Theme>
    );
    return onConfirm;
  }

  it('renders its portal inside a theme of the page appearance, so its gaps and colours resolve', () => {
    renderDialog();

    const actions = screen.getByRole('button', { name: 'Revoke' }).closest('.confirm-dialog__actions');
    const theme = actions?.closest('.radix-themes');
    expect(theme).not.toBeNull();
    expect(theme).toHaveClass('dark');
  });

  it('confirms with the confirm button', () => {
    const onConfirm = renderDialog();

    fireEvent.click(screen.getByRole('button', { name: 'Revoke' }));

    expect(onConfirm).toHaveBeenCalled();
  });
});
