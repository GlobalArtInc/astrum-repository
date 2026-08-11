

import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { SystemAlerts } from '../SystemAlerts';

// Mutable flag the useIsPreviewUI mock reads so each test can toggle the UI mode.
// Prefixed with `mock` so jest allows referencing it inside the factory.
let mockPreviewUI = true;
jest.mock('../../Navigation', () => ({
  useIsPreviewUI: () => mockPreviewUI,
}));

// Stub the child alert so this test isolates the host's gating/composition.
jest.mock('../RecoveryModeAlert', () => ({
  RecoveryModeAlert: () => <div data-testid="recovery-mode-alert" />,
}));

const renderInTheme = (ui: React.ReactElement) => render(<Theme>{ui}</Theme>);

describe('SystemAlerts', () => {
  it('renders the host with its alert children in the Preview UI', () => {
    mockPreviewUI = true;
    renderInTheme(<SystemAlerts />);
    expect(screen.getByTestId('as-system-alerts')).toBeInTheDocument();
    expect(screen.getByTestId('recovery-mode-alert')).toBeInTheDocument();
  });

  it('renders nothing outside the Preview UI', () => {
    mockPreviewUI = false;
    const { container } = renderInTheme(<SystemAlerts />);
    expect(screen.queryByTestId('as-system-alerts')).not.toBeInTheDocument();
    // Only the Radix Theme wrapper remains, no host box.
    expect(container.querySelector('.as-system-alerts')).toBeNull();
  });

  it('appends a custom className to the host', () => {
    mockPreviewUI = true;
    renderInTheme(<SystemAlerts className="extra-class" />);
    const host = screen.getByTestId('as-system-alerts');
    expect(host).toHaveClass('as-system-alerts');
    expect(host).toHaveClass('extra-class');
  });
});
