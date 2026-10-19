import { Theme } from '@radix-ui/themes';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it } from 'vitest';
import { Tooltip } from './Tooltip';

function hint() {
  return render(
    <Theme>
      <Tooltip content="Language" delayDuration={0}>
        <button type="button">Switch</button>
      </Tooltip>
    </Theme>,
  );
}

afterEach(() => {
  delete document.documentElement.dataset.input;
});

describe('the tooltip', () => {
  it('stays closed when focus comes back to its trigger after a pointer was used', async () => {
    hint();
    document.documentElement.dataset.input = 'pointer';
    act(() => screen.getByRole('button', { name: 'Switch' }).focus());
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(screen.queryByRole('tooltip')).toBeNull();
  });

  it('opens on hover and on keyboard focus', async () => {
    hint();
    document.documentElement.dataset.input = 'pointer';
    await userEvent.hover(screen.getByRole('button', { name: 'Switch' }));
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Language');
    await userEvent.unhover(screen.getByRole('button', { name: 'Switch' }));
    document.documentElement.dataset.input = 'keyboard';
    act(() => screen.getByRole('button', { name: 'Switch' }).blur());
    act(() => screen.getByRole('button', { name: 'Switch' }).focus());
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Language');
  });
});
