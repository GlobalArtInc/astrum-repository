import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { renderWithApp } from '../test/render';
import { QueryTabs } from './QueryTabs';

const TABS = [
  { value: 'overview', label: 'Overview', content: <p>overview body</p> },
  { value: 'usage', label: 'Usage', content: <p>usage body</p> },
];

describe('tabs in the address', () => {
  it('open the tab named by ?tab= and write the chosen one', async () => {
    const { router } = renderWithApp(<QueryTabs tabs={TABS} />, { path: '/browse/raw?tab=usage' });
    expect(screen.getByText('usage body')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('tab', { name: /Overview/ }));
    expect(screen.getByText('overview body')).toBeInTheDocument();
    expect(router.state.location.search).toBe('');
    await userEvent.click(screen.getByRole('tab', { name: /Usage/ }));
    expect(router.state.location.search).toBe('?tab=usage');
  });

  it('fall back to the first tab for unknown names', () => {
    renderWithApp(<QueryTabs tabs={TABS} />, { path: '/x?tab=nope' });
    expect(screen.getByRole('tab', { name: /Overview/ })).toHaveAttribute('aria-selected', 'true');
  });
});
