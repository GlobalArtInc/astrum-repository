
import React from 'react';
import { when } from 'jest-when';
import { render, screen, within } from '@testing-library/react';
import UIStrings from '../../../constants/UIStrings';
import {
  assertCommunityEditionLimitMessageShowing
} from './SystemNotices.testutils';
import RecoveryModeAlert from './RecoveryModeAlert/RecoveryModeAlert';

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    state: jest.fn().mockReturnValue({
      getValue: jest.fn()
    }),
    usePermission: jest.fn(),
    useState: jest.fn(),
    useUser: jest.fn().mockReturnValue({ administrator: true })
  },
}));

jest.mock('@uirouter/react', () => ({
  ...jest.requireActual('@uirouter/react'),
  useRouter: jest.fn(() => ({
    stateService: {
      go: jest.fn()
    }
  }))
}));

const {RECOVERY_MODE_ALERT: {LABEL, TEXT}} = UIStrings;

describe('SystemNotices - Recovery Mode Alert', () => {
  const ExtJS = require('@globalart/astrum-ui-plugin').ExtJS;

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders the recovery mode alert when enabled', () => {
    givenRecoveryModeEnabled(true);
    renderComponent();

    const alert = assertCommunityEditionLimitMessageShowing(
      'While Recovery Mode is on, data repair conflicting tasks are blocked to protect data consistency.',
      LABEL
    );

    // Verify it's non-dismissable
    expect(within(alert).queryByRole('button', { name: 'Close' })).not.toBeInTheDocument();
  });

  it('does not render when recovery mode is disabled', () => {
    givenRecoveryModeEnabled(false);
    renderComponent();

    expect(screen.queryByRole('complementary', { name: 'alert system notice' })).not.toBeInTheDocument();
  });

  it('does not render when user is not admin', () => {
    ExtJS.useUser.mockReturnValue({ administrator: false });
    givenRecoveryModeEnabled(true);
    renderComponent();

    expect(screen.queryByRole('complementary', { name: 'alert system notice' })).not.toBeInTheDocument();
  });

  function renderComponent() {
    return render(<RecoveryModeAlert />);
  }

  function givenRecoveryModeEnabled(enabled) {
    when(ExtJS.state().getValue)
      .calledWith('recovery.mode.enabled')
      .mockReturnValue(enabled);
    when(ExtJS.useState)
      .calledWith(expect.any(Function))
      .mockImplementation((fn) => fn());
    when(ExtJS.usePermission)
      .calledWith(expect.any(Function), [true])
      .mockReturnValue(true);
  }
});
