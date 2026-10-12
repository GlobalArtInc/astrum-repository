import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { SystemAlerts } from '../SystemAlerts';

// Stub the child alert so this test isolates the host's composition.
jest.mock('../RecoveryModeAlert', () => ({
  RecoveryModeAlert: () => <div data-testid="recovery-mode-alert" />,
}));

const renderInTheme = (ui: React.ReactElement) => render(<Theme>{ui}</Theme>);

describe('SystemAlerts', () => {
  it('renders the host with its alert children', () => {
    renderInTheme(<SystemAlerts />);
    expect(screen.getByTestId('as-system-alerts')).toBeInTheDocument();
    expect(screen.getByTestId('recovery-mode-alert')).toBeInTheDocument();
  });

  it('appends a custom className to the host', () => {
    renderInTheme(<SystemAlerts className="extra-class" />);
    const host = screen.getByTestId('as-system-alerts');
    expect(host).toHaveClass('as-system-alerts');
    expect(host).toHaveClass('extra-class');
  });
});
