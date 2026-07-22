
import React from 'react';
import {render, screen} from '@testing-library/react';
import {when} from 'jest-when';
import {ExtJS} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import UpgradeAlert from './UpgradeAlert';
import {UpgradeAlertFunctions} from '../../../widgets/SystemStatusAlerts/UpgradeAlert/UpgradeAlertHelper';
const {UPGRADE_ALERT: {PENDING, WARN}} = UIStrings;

jest.mock('axios', () => ({
  ...jest.requireActual('axios'),
  get: jest.fn()
}));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    state: jest.fn().mockReturnValue({
      getValue: jest.fn(),
    }),
    useState: jest.fn(),
    usePermission: jest.fn(),
  },
}));

describe('Upgrade Alert', () => {
  async function renderView(currentState, onClose = null) {
    when(ExtJS.state().getValue)
        .calledWith('dbUpgrade')
        .mockReturnValue({currentState: currentState});

    when(ExtJS.useState)
        .calledWith(UpgradeAlertFunctions.hasUser)
        .mockReturnValue('true');
        
    when(ExtJS.useState)
        .calledWith(UpgradeAlertFunctions.currentState)
        .mockReturnValue(currentState);

    when(ExtJS.usePermission)
        .calledWith(UpgradeAlertFunctions.checkPermissions)
        .mockReturnValue(true);

    return render(<UpgradeAlert onClose={onClose}/>);
  };

  it('renders the status upgrade alert for pending upgrade', async () => {
    const {container} = await renderView('needsUpgrade');
    const alert = container.querySelector('.nx-alert');
    const alertMessage = PENDING.STATUS_LABEL + ' ' + PENDING.TEXT;

    expect(alert).toBeInTheDocument();
    expect(alert).toHaveTextContent(alertMessage);
    expect(screen.getByRole('button', {name: 'Finalize Upgrade'})).toBeInTheDocument();
  });

  it('renders the status upgrade alert for node version mismatch', async () => {
    const onClose = jest.fn();

    const {container} = await renderView('versionMismatch', onClose);
    const alert = container.querySelector('.nx-alert');
    const alertMessage = WARN.LABEL + ' ' + WARN.TEXT;

    expect(alert).toBeInTheDocument();
    expect(alert).toHaveTextContent(alertMessage);
    expect(screen.getByRole('button', {name: 'Close'})).toBeInTheDocument();
  });


  it('renders the status upgrade alert for older than 1 year', async () => {
    const {container} = await renderView('astrumNeedsUpgrade1Year');

    const alert = container.querySelector('.nx-alert');
    const alertMessage = PENDING.OLDER_LABEL + ' You are currently running a GlobalArt Astrum Repository version that is ' +
      'in extended maintenance. See our Sunsetting information for details about our product development lifecycle. ' +
      'Also see our release notes to learn more about the newest Astrum Repository version available for your instance.';

    expect(alert).toBeInTheDocument();
    expect(alert).toHaveTextContent(alertMessage);
  });

  it('renders the status upgrade alert for older than 18 months', async () => {
    const {container} = await renderView('astrumNeeds18MonthsUpgrade');

    const alert = container.querySelector('.nx-alert');
    const alertMessage = PENDING.OLDER_LABEL + ' You are currently running a sunsetted GlobalArt Astrum Repository version. ' +
      'See our Sunsetting information for details about our product development lifecycle. Also see our release notes to ' +
      'learn more about the newest Astrum Repository version available for your instance.';

    expect(alert).toBeInTheDocument();
    expect(alert).toHaveTextContent(alertMessage);
  });
});
