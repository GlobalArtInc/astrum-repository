
import React from 'react';
import {fireEvent, waitForElementToBeRemoved} from '@testing-library/react';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';
import axios from 'axios';

import LoggingConfigurationList from './LoggingConfigurationList';

import UIStrings from '../../../../constants/UIStrings';
import { UIRouter } from '@uirouter/react';
import { getRouter } from '../../../../routerConfig/routerConfig';

jest.mock('axios', () => ({
  ...jest.requireActual('axios'), // Use most functions from actual axios
  get: jest.fn()
}));

describe('LoggingConfigurationList', function() {
  function renderView() {
    const router = getRouter();
    const view = (
      <UIRouter router={router}>
        <LoggingConfigurationList/>
      </UIRouter>
    );
    return TestUtils.render(view, ({queryByPlaceholderText}) => ({
      filter: () => queryByPlaceholderText(UIStrings.LOGGING.FILTER_PLACEHOLDER)
    }));
  }

  it('renders the resolved data', async function() {
    const rows = [
      {name: 'ROOT', level: 'INFO'},
      {name: 'org.globalart.astrum', level: 'DEBUG'}
    ];

    axios.get.mockReturnValue(Promise.resolve({
      data: rows
    }));

    const {container, loadingMask} = renderView();

    await waitForElementToBeRemoved(loadingMask);

    rows.forEach((row, i) => {
      expect(container.querySelector(`tbody tr:nth-child(${i+1}) td:nth-child(1)`)).toHaveTextContent(row.name);
      expect(container.querySelector(`tbody tr:nth-child(${i+1}) td:nth-child(2)`)).toHaveTextContent(row.level);
    });
  });

  it('renders an error message', async function() {
    axios.get.mockReturnValue(Promise.reject({message: 'Error'}));

    const {container, loadingMask} = renderView();

    await waitForElementToBeRemoved(loadingMask);

    expect(container.querySelector('.nx-cell--meta-info')).toHaveTextContent('Error');
  });

  it('searches for lower-cased strings in the logger names', async function() {
    axios.get.mockReturnValue(Promise.resolve({
      data: [
        {name: 'ROOT', level: 'INFO'},
        {name: 'org.globalart.astrum', level: 'DEBUG'}
      ]
    }));

    const {container, loadingMask, filter} = renderView();

    await waitForElementToBeRemoved(loadingMask);

    fireEvent.change(filter(), {target: {value: 'globalart'}});

    expect(container.querySelector('tbody tr:nth-child(1) td:nth-child(1)')).toHaveTextContent('org.globalart.astrum');
  });
});
