
import React from 'react';
import Axios from 'axios';
import {act} from 'react-dom/test-utils';
import userEvent from '@testing-library/user-event';

import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import UIStrings from '../../../../constants/UIStrings';
import HealthCheckEula from './HealthCheckEula';

jest.mock('axios', () => {
  return {
    put: jest.fn(() => Promise.resolve()),
  };
});

jest.mock('@globalart/astrum-ui-plugin', () => {
  return {
    ...jest.requireActual('@globalart/astrum-ui-plugin'),
    ExtJS: {
      urlOf: jest.fn()
    }
  }
});

describe('HealthCheckEula', () => {
  const render = () => TestUtils.render(<HealthCheckEula onAccept={() => {}} onDecline={() => {}}/>, ({getByText}) => ({
    acceptButton: () => getByText(UIStrings.HEALTHCHECK_EULA.BUTTONS.ACCEPT),
    declineButton: () => getByText(UIStrings.HEALTHCHECK_EULA.BUTTONS.DECLINE),
  }));

  it('sends the accept eula message', async () => {
    let {acceptButton} = render();

    await act(async () => userEvent.click(acceptButton()));

    expect(Axios.put).toHaveBeenCalledTimes(1);
    expect(Axios.put).toHaveBeenCalledWith(`service/rest/internal/ui/healthcheck/eula`);
  });

  it('does not send accept eula message', async () => {
    let {declineButton} = render();

    await act(async () => userEvent.click(declineButton()));

    expect(Axios.put).toHaveBeenCalledTimes(0);
  });
});
