import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';

const mockGetValue = jest.fn();

jest.mock('../../../../../interface/ExtJS', () => ({
  ExtJS: {
    checkPermission: jest.fn().mockReturnValue(false),
    state: () => ({
      getValue: (...args) => mockGetValue(...args),
      getUser: () => null,
    }),
  },
}));

jest.mock('@xstate/react', () => ({
  useMachine: () => [{ context: { showConnectAction: true }, matches: () => false }, jest.fn()],
}));

import OutreachActions from '../OutreachActions';

describe('OutreachActions', () => {
  beforeEach(() => mockGetValue.mockReset());

  it('renders without browseable formats in the state, as on a first visit of the dashboard', () => {
    mockGetValue.mockImplementation((key, defaultValue) => defaultValue);

    render(<Theme><OutreachActions /></Theme>);

    expect(screen.queryByText('Browse')).not.toBeInTheDocument();
  });

  it('offers browsing once there are browseable formats', () => {
    mockGetValue.mockImplementation((key, defaultValue) => (key === 'browseableformats' ? ['maven2'] : defaultValue));

    const { container } = render(<Theme><OutreachActions /></Theme>);

    expect(container.querySelectorAll('.as-quick-action-chevron').length).toBeGreaterThan(0);
  });
});
