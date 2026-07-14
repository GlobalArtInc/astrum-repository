

import { when } from 'jest-when';
import { ExtJS } from '@globalart/astrum-ui-plugin';
import { UpgradeAlertFunctions } from './UpgradeAlert/UpgradeAlertHelper';
import { render, screen, within } from '@testing-library/react';
import SystemNotices from './SystemNotices';
import React from 'react';
import userEvent from '@testing-library/user-event';


export function renderView(gracePeriodEnd, throttlingStatus, currentState, message, recoveryModeEnabled) {
  givenInitialState(gracePeriodEnd, throttlingStatus, currentState, message, recoveryModeEnabled)

  return render(<SystemNotices onClose={jest.fn()} />);
}

export function givenInitialState(gracePeriodEnd, throttlingStatus, currentState, message, recoveryModeEnabled) {
  givenGetValue(currentState, message, recoveryModeEnabled, throttlingStatus, gracePeriodEnd)
  givenUseState(throttlingStatus, gracePeriodEnd, currentState, message);

  when(ExtJS.usePermission)
    .calledWith(UpgradeAlertFunctions.checkPermissions)
    .mockReturnValue(true);

  // Mock NX.State.getUser to return a user so hasUser() returns true
  global.NX.State.getUser.mockReturnValue({ username: 'test-user' });
}

function givenUseState(throttlingStatus, gracePeriodEnd, currentState, message) {
  when(ExtJS.useState)
      .calledWith(UpgradeAlertFunctions.hasUser)
      .mockReturnValue(true);

  when(ExtJS.useState)
      .calledWith(UpgradeAlertFunctions.currentState)
      .mockReturnValue(currentState);

  when(ExtJS.useState)
      .calledWith(UpgradeAlertFunctions.message)
      .mockReturnValue(message);

  when(ExtJS.useState)
      .calledWith(expect.any(Function))
      .mockImplementation((fn) => fn());

}

function givenGetValue(currentState, message, recoveryModeEnabled, throttlingStatus, gracePeriodEnd) {
  when(ExtJS.state().getValue)
      .calledWith('dbUpgrade', expect.anything())
      .mockReturnValue({ currentState, message });
  when(ExtJS.state().getValue)
      .calledWith('astrum.node.id')
      .mockReturnValue('node-example-id');
  when(ExtJS.state().getValue)
      .calledWith('astrum.datastore.clustered.enabled')
      .mockReturnValue(false);
  when(ExtJS.state().getValue)
      .calledWith('astrum.malware.count')
      .mockReturnValue({ totalCount: 3 });
  when(ExtJS.state().getValue)
      .calledWith('recovery.mode.enabled')
      .mockReturnValue(recoveryModeEnabled);
  when(ExtJS.state().getValue)
      .calledWith('astrum.community.throttlingStatus')
      .mockReturnValue(throttlingStatus);
  when(ExtJS.state().getValue)
      .calledWith('astrum.community.gracePeriodEnds')
      .mockReturnValue(gracePeriodEnd);
}


export function assertCommunityEditionLimitMessageShowing(message, title) {
  const alert = screen.getByRole('complementary', { name: 'alert system notice'});
  expect(alert).toBeVisible();

  if (title) {
    expect(within(alert).getByRole('heading', {name: title})).toBeVisible();
  }

  expect(within(alert).getByText(message, { exact: false })).toBeVisible()

  return alert;
}

export async function assertCanDismissTheBanner(alert) {
  const closeButton = within(alert).getByRole('button', { name: 'Close' });
  expect(closeButton).toBeVisible();
  await userEvent.click(closeButton);

  expect(screen.queryByRole('complementary', { name: 'alert system notice'})).not.toBeInTheDocument();
}

export function assertHasLinkWithText(alert, text, href) {
  const lnk = within(alert).getByRole('link', {name: text});
  expect(lnk).toBeVisible();
  expect(lnk).toHaveAttribute('href', href);
}
