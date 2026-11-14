import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { securityArea } from '../areas/security';
import { ADMIN_STATE, ANONYMOUS_STATE, renderWithApp } from '../test/render';
import { useVisibleSettings } from './registry';

function VisibleTiles() {
  const tiles = useVisibleSettings();
  return <p>{tiles.map((tile) => tile.id).join(' ')}</p>;
}

describe('the settings tiles', () => {
  it('hides a tile that requires a signed-in user from anonymous visitors', () => {
    renderWithApp(<VisibleTiles />, { state: ANONYMOUS_STATE, areas: [securityArea] });
    expect(screen.queryByText(/share-links/)).toBeNull();
  });

  it('shows that tile to a signed-in user', () => {
    renderWithApp(<VisibleTiles />, { state: ADMIN_STATE, areas: [securityArea] });
    expect(screen.getByText(/share-links/)).toBeInTheDocument();
  });
});
