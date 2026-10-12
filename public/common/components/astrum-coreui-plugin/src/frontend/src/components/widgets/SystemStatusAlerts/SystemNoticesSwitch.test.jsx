
import React from 'react';
import {render, screen} from '@testing-library/react';
import '@testing-library/jest-dom';

import SystemNoticesSwitch from './SystemNoticesSwitch';

jest.mock('@globalart/astrum-ui-plugin', () => ({
  SystemAlerts: () => <div data-testid="system-alerts" />,
}));

jest.mock('./SystemNotices', () => ({
  __esModule: true,
  default: () => <div data-testid="system-notices" />,
}));

describe('SystemNoticesSwitch', () => {
  it('renders SystemAlerts and SystemNotices', () => {
    render(<SystemNoticesSwitch />);
    expect(screen.getByTestId('system-alerts')).toBeInTheDocument();
    expect(screen.getByTestId('system-notices')).toBeInTheDocument();
  });
});
