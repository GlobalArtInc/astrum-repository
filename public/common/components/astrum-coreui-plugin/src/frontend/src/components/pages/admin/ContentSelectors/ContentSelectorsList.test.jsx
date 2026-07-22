
import React from 'react';
import {waitForElementToBeRemoved, screen} from '@testing-library/react';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';
import axios from 'axios';
import {ExtJS} from '@globalart/astrum-ui-plugin';
import {when} from 'jest-when';

import ContentSelectorsList from './ContentSelectorsList';

import UIStrings from '../../../../constants/UIStrings';
import {UIRouter} from '@uirouter/react';
import {getRouter} from '../../../../routerConfig/routerConfig';

jest.mock('axios', () => ({
  get: jest.fn()
}));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    checkPermission: jest.fn(),
    state: jest.fn().mockReturnValue({
      getValue: jest.fn()
    })
  }
}));

describe('ContentSelectorsList', function() {
  function renderView() {
    const router = getRouter();
    const view = (
      <UIRouter router={router}>
        <ContentSelectorsList />
      </UIRouter>
    )
    return TestUtils.render(view, ({queryByPlaceholderText}) => ({
      filter: () => queryByPlaceholderText(UIStrings.CONTENT_SELECTORS.FILTER_PLACEHOLDER)
    }));
  }

  it('renders the resolved data', async function() {
    const rows = [
      {
        name: 'an xss<img src="/static/rapture/resources/icons/x16/user.png" onload="alert(0)">',
        type: 'csel',
        description: 'description'
      },
      {
        name: 'jexl-selector',
        type: 'jexl',
        description: 'jexl-description'
      }
    ];

    axios.get.mockReturnValue(Promise.resolve({
      data: rows
    }));

    const {container, loadingMask} = renderView();

    await waitForElementToBeRemoved(loadingMask);

    rows.forEach((row, i) => {
      expect(container.querySelector(`tbody tr:nth-child(${i+1}) td:nth-child(1)`)).toHaveTextContent(row.name);
      expect(container.querySelector(`tbody tr:nth-child(${i+1}) td:nth-child(2)`)).toHaveTextContent(row.type.toUpperCase());
      expect(container.querySelector(`tbody tr:nth-child(${i+1}) td:nth-child(3)`)).toHaveTextContent(row.description);
    });
  });

  it('renders an error message', async function() {
    axios.get.mockReturnValue(Promise.reject({message: 'Error'}));

    const {container, loadingMask} = renderView();

    await waitForElementToBeRemoved(loadingMask);

    expect(container.querySelector('.nx-cell--meta-info')).toHaveTextContent('Error');
  });

  it('disables create button without the permissions', async function() {
    when(ExtJS.checkPermission)
    .calledWith('astrum:selectors:create')
    .mockReturnValue(false);

    const {container, loadingMask} = renderView();

    await waitForElementToBeRemoved(loadingMask);

    expect(container.querySelector('button', {text: 'Create Selector'})).toBeDisabled();
  });

  describe('Analytics IDs', function() {
    it('has as-content-selector-create analytics ID on create button', async function() {
      when(ExtJS.checkPermission)
        .calledWith('astrum:selectors:create')
        .mockReturnValue(true);

      axios.get.mockReturnValue(Promise.resolve({data: []}));

      const {loadingMask} = renderView();

      await waitForElementToBeRemoved(loadingMask);

      const createButton = screen.getByRole('button', {name: /create selector/i});
      expect(createButton).toHaveAttribute('data-analytics-id', 'as-content-selector-create');
    });

    it('has as-content-selector-filter analytics ID on filter input', async function() {
      axios.get.mockReturnValue(Promise.resolve({data: []}));

      const {loadingMask} = renderView();

      await waitForElementToBeRemoved(loadingMask);

      const filterInput = screen.getByPlaceholderText(UIStrings.CONTENT_SELECTORS.FILTER_PLACEHOLDER);
      expect(filterInput).toHaveAttribute('data-analytics-id', 'as-content-selector-filter');
    });
  });

  describe('Accessibility', function() {
    it('has aria-label on filter input', async function() {
      axios.get.mockReturnValue(Promise.resolve({data: []}));

      const {loadingMask} = renderView();

      await waitForElementToBeRemoved(loadingMask);

      const filterInput = screen.getByPlaceholderText(UIStrings.CONTENT_SELECTORS.FILTER_PLACEHOLDER);
      expect(filterInput).toHaveAttribute('aria-label', 'Filter content selectors by name');
    });
  });
});
