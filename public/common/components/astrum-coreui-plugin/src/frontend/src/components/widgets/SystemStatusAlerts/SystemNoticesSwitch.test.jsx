

import React from 'react';
import {render, screen} from '@testing-library/react';
import '@testing-library/jest-dom';

import SystemNoticesSwitch from './SystemNoticesSwitch';

// Mutable flag the useIsPreviewUI mock reads so each test can toggle the UI mode.
let previewUI = false;

jest.mock('@globalart/astrum-ui-plugin', () => ({
  useIsPreviewUI: () => previewUI,
  SystemAlerts: () => <div data-testid="system-alerts" />,
}));

// SystemNotices records the isPreviewUI prop it was rendered with so we can
// assert the switch forwards it correctly.
jest.mock('./SystemNotices', () => ({
  __esModule: true,
  default: ({isPreviewUI = false} = {}) => (
    <div data-testid="system-notices" data-preview={String(isPreviewUI)} />
  ),
}));

describe('SystemNoticesSwitch', () => {
  it('classic UI: renders SystemNotices(isPreviewUI=false), not SystemAlerts', () => {
    previewUI = false;
    render(<SystemNoticesSwitch />);
    expect(screen.getByTestId('system-notices')).toHaveAttribute('data-preview', 'false');
    expect(screen.queryByTestId('system-alerts')).not.toBeInTheDocument();
  });

  it('preview UI: renders SystemAlerts + SystemNotices(isPreviewUI=true)', () => {
    previewUI = true;
    render(<SystemNoticesSwitch />);
    expect(screen.getByTestId('system-alerts')).toBeInTheDocument();
    expect(screen.getByTestId('system-notices')).toHaveAttribute('data-preview', 'true');
  });
});
