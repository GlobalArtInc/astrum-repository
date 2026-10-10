
import React from 'react';
import axios from 'axios';
import {fireEvent, waitFor, waitForElementToBeRemoved} from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import {ExtJS} from '@globalart/astrum-ui-plugin';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import UIStrings from '../../../../constants/UIStrings';

import LoggingConfigurationForm from './LoggingConfigurationForm';
import {UIRouter, useCurrentStateAndParams} from "@uirouter/react";
import { getRouter } from '../../../../routerConfig/routerConfig';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';

jest.mock('axios', () => ({
  ...jest.requireActual('axios'), // Use most functions from actual axios
  get: jest.fn(),
  put: jest.fn(),
  post: jest.fn()
}));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    requestConfirmation: jest.fn(),
    state: jest.fn().mockReturnValue({
      getValue: jest.fn()
    })
  },
  FormUtils: {
    buildFormMachine: function(args) {
      const machine = jest.requireActual('@globalart/astrum-ui-plugin').FormUtils.buildFormMachine(args);
      return machine.withConfig({
        actions: {
          logSaveSuccess: jest.fn(),
          logLoadError: jest.fn()
        }
      })
    },
    formProps: jest.requireActual('@globalart/astrum-ui-plugin').FormUtils.formProps,
    fieldProps: jest.requireActual('@globalart/astrum-ui-plugin').FormUtils.fieldProps,
    saveTooltip: jest.requireActual('@globalart/astrum-ui-plugin').FormUtils.saveTooltip,
    discardTooltip: jest.requireActual('@globalart/astrum-ui-plugin').FormUtils.discardTooltip
  }
}));

let stateServiceGoMock = jest.fn();

jest.mock('@uirouter/react', () => ({
  ...jest.requireActual('@uirouter/react'),
  useCurrentStateAndParams: jest.fn(),
  useRouter: () =>({
    stateService: {
      go: stateServiceGoMock,
    }
  })
}));

describe('LoggingConfigurationForm', function() {
  const CONFIRM = Promise.resolve();
  const OVERRIDDEN_LOGGER = Promise.resolve({data: {override: true}});

  function renderEditView(itemId = 'ROOT') {
    useCurrentStateAndParams.mockReturnValue({
      state: {name: ROUTE_NAMES.ADMIN.SUPPORT.LOGGING.CREATE},
      params: {itemId}
    });

    const router = getRouter();
    const view = (
        <UIRouter router={router}>
          <LoggingConfigurationForm />
        </UIRouter>
    );
    return renderView(view);
  }

  function renderView(view) {
    return TestUtils.render(view, ({queryByLabelText, queryByText}) => ({
      name: () => queryByLabelText(UIStrings.LOGGING.NAME_LABEL),
      level: () => queryByLabelText(UIStrings.LOGGING.LEVEL_LABEL),
      saveButton: () => queryByText(UIStrings.SETTINGS.SAVE_BUTTON_LABEL),
      cancelButton: () => queryByText(UIStrings.SETTINGS.CANCEL_BUTTON_LABEL),
      resetMask: () => queryByText(UIStrings.LOGGING.MESSAGES.RESETTING),
      resetButton: () => queryByText(UIStrings.LOGGING.RESET_BUTTON)
    }));
  }

  const changeFieldAndAssertValue = async (fieldSelector, value) => {
    fireEvent.change(fieldSelector(), {target: {value: value}});
    await waitFor(() => expect(fieldSelector()).toHaveValue(value));
  };

  it('renders the resolved data', async function() {
    axios.get.mockResolvedValue({
      data: {name: 'ROOT', level: 'INFO'}
    });

    const {loadingMask, name, level} = renderEditView();

    await waitForElementToBeRemoved(loadingMask);

    expect(name()).toHaveValue('ROOT');
    expect(level()).toHaveValue('INFO');
  });

  it('renders an error message', async function() {
    axios.get.mockImplementation(() => new Promise((resolve, reject) => {
      setTimeout(() => reject({message: 'Error'}), 100);
    }));

    const {container, loadingMask} = renderEditView();

    await waitForElementToBeRemoved(loadingMask);

    expect(container.querySelector('.as-alert--error')).toHaveTextContent('Error');
  });

  it('fires onDone when cancelled', async function() {
    const {loadingMask, cancelButton} = renderEditView('');

    await waitForElementToBeRemoved(loadingMask);

    userEvent.click(cancelButton());

    // expect onDone is called
    expect(stateServiceGoMock).toHaveBeenCalledWith(ROUTE_NAMES.ADMIN.SUPPORT.LOGGING.LIST);
  });

  it('requires the name field when creating a new logging configuration', async function() {
    const {loadingMask, name, level, saveButton} = renderEditView('');
    await waitForElementToBeRemoved(loadingMask);

    expect(level()).toHaveValue('INFO');

    userEvent.click(saveButton())
    expect(name()).toHaveAccessibleErrorMessage(TestUtils.REQUIRED_MESSAGE);

    await changeFieldAndAssertValue(name, 'name');
    expect(name()).not.toHaveAccessibleErrorMessage(TestUtils.REQUIRED_MESSAGE);
  });

  it('requests confirmation when the logger is overridden and saves when requested', async function() {
    const {loadingMask, name, level, saveButton} = renderEditView('');

    await waitForElementToBeRemoved(loadingMask);

    axios.put.mockReturnValue(Promise.resolve());

    await changeFieldAndAssertValue(name, 'name');
    await changeFieldAndAssertValue(level, 'DEBUG');
    expect(saveButton()).not.toHaveClass('disabled');

    axios.get.mockReturnValue(OVERRIDDEN_LOGGER);
    ExtJS.requestConfirmation.mockReturnValue(CONFIRM);
    userEvent.click(saveButton());

    await waitFor(() => expect(axios.put).toBeCalledWith(
        '/service/rest/internal/ui/loggingConfiguration/name',
        {name: 'name', level: 'DEBUG'}
    ));
  });

  it('saves', async function() {
    axios.get.mockReturnValue(Promise.resolve({
      data: {name: 'ROOT', level: 'INFO'}
    }));

    const {loadingMask, level, saveButton} = renderEditView();

    await waitForElementToBeRemoved(loadingMask);

    await waitFor(() => expect(window.dirty).toEqual([]));

    await changeFieldAndAssertValue(level, 'DEBUG');

    await waitFor(() => expect(window.dirty).toEqual(['LoggingConfigurationFormMachine']));

    userEvent.click(saveButton());

    await waitFor(() => expect(axios.put).toHaveBeenLastCalledWith(
        '/service/rest/internal/ui/loggingConfiguration/ROOT',
        {name: 'ROOT', level: 'DEBUG'}
    ));
    expect(window.dirty).toEqual([]);
  });

  it('resets an edited logger', async function() {
    axios.get.mockReturnValue(Promise.resolve({
      data: {name: 'test', level: 'ERROR'}
    }));

    const {loadingMask, resetButton, resetMask} = renderEditView();

    await waitForElementToBeRemoved(loadingMask);

    ExtJS.requestConfirmation.mockReturnValue(CONFIRM);
    axios.post.mockReturnValue(CONFIRM);
    userEvent.click(resetButton());

    await waitFor(() => expect(resetMask()).toBeInTheDocument());

    expect(axios.post).toBeCalledWith('/service/rest/internal/ui/loggingConfiguration/test/reset');
  });
});
