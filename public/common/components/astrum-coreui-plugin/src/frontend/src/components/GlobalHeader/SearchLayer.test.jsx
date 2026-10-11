import React from 'react';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';

let transitionSuccess;

jest.mock('@uirouter/react', () => ({
  useTransitionHook: (_hook, _criteria, callback) => {
    transitionSuccess = callback;
  },
}));

jest.mock('./SearchRadix', () => ({ isPreviewUI }) => (
  <input aria-label="Search components" data-preview={String(isPreviewUI)} />
));

import SearchLayer from './SearchLayer';

function renderLayer(onClose = jest.fn()) {
  render(
    <Theme>
      <button type="button">opener</button>
      <SearchLayer onClose={onClose} />
    </Theme>
  );
  return onClose;
}

describe('SearchLayer', () => {
  it('holds the search field of the header, of Astrum One, with the focus', () => {
    renderLayer();

    const field = screen.getByRole('textbox', { name: 'Search components' });
    expect(screen.getByRole('dialog', { name: 'Search components' })).toBeInTheDocument();
    expect(field).toHaveAttribute('data-preview', 'true');
    expect(field).toHaveFocus();
  });

  it('closes with its back button', () => {
    const onClose = renderLayer();

    fireEvent.click(screen.getByRole('button', { name: 'Close search' }));

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('closes on Esc', () => {
    const onClose = renderLayer();

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('closes once a search or a suggestion was opened', () => {
    const onClose = renderLayer();

    act(() => transitionSuccess());

    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
