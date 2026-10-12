import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Link } from 'react-router';
import { describe, expect, it } from 'vitest';
import { renderWithApp } from '../test/render';
import { UnsavedChangesGuard } from './UnsavedChangesGuard';

function Editor() {
  return (
    <>
      <UnsavedChangesGuard when />
      <Link to="/elsewhere">away</Link>
    </>
  );
}

describe('the unsaved changes guard', () => {
  it('asks before leaving and stays or leaves', async () => {
    const { router } = renderWithApp(null, {
      path: '/edit',
      routes: [
        { path: '/edit', element: <Editor /> },
        { path: '/elsewhere', element: <p>elsewhere</p> },
      ],
    });
    await userEvent.click(screen.getByRole('link', { name: 'away' }));
    expect(await screen.findByRole('alertdialog', { name: 'Leave without saving?' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Stay' }));
    expect(router.state.location.pathname).toBe('/edit');
    await userEvent.click(screen.getByRole('link', { name: 'away' }));
    await userEvent.click(await screen.findByRole('button', { name: 'Leave' }));
    expect(await screen.findByText('elsewhere')).toBeInTheDocument();
  });
});
