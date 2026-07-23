
import React from 'react';
import Axios from 'axios';
import {screen, waitFor} from '@testing-library/react';
import {act} from 'react-dom/test-utils';
import userEvent from '@testing-library/user-event';

import {ExtJS} from '@globalart/astrum-ui-plugin';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import UIStrings from '../../../../constants/UIStrings';
import PasswordChangeForm from './PasswordChangeForm';
import { when } from 'jest-when';

jest.mock('@globalart/astrum-ui-plugin', () => {
  return {
    ...jest.requireActual('@globalart/astrum-ui-plugin'),
    ExtJS: {
      showSuccessMessage: jest.fn(),
      showErrorMessage: jest.fn(),
      fetchAuthenticationToken: jest.fn(() => Promise.resolve({data: 'fakeToken'})),
      checkPermission: jest.fn(),
    }
  };
});

jest.mock('axios', () => {
  return {
    put: jest.fn(() => Promise.resolve()),
  };
});

const selectors = {
  ...TestUtils.formSelectors,
  querySubmitButton: () => screen.queryByRole('button', {name: 'Change password'})
};

describe('PasswordChangeForm', () => {
  const CHANGE_PASSWORD_URL = '/service/rest/internal/ui/user/admin/password';

  const render = () => TestUtils.render(<PasswordChangeForm userId="admin"/>, ({queryByLabelText, queryByText}) => ({
    passwordCurrent: () => queryByLabelText(UIStrings.USER_ACCOUNT.PASSWORD_CURRENT_FIELD_LABEL),
    passwordNew: () => queryByLabelText(UIStrings.USER_ACCOUNT.PASSWORD_NEW_FIELD_LABEL),
    passwordNewConfirm: () => queryByLabelText(UIStrings.USER_ACCOUNT.PASSWORD_NEW_CONFIRM_FIELD_LABEL),
    changePasswordButton: () => queryByText(UIStrings.USER_ACCOUNT.ACTIONS.changePassword),
    discardButton: () => queryByText(UIStrings.USER_ACCOUNT.ACTIONS.discardChangePassword),
  }));

  beforeEach(() => {
    when(ExtJS.checkPermission)
    .calledWith('astrum:userschangepw:create')
    .mockReturnValue(true);
  });

  it('renders correctly', async () => {
    let {passwordCurrent, passwordNew, passwordNewConfirm, discardButton} = render();

    expect(passwordCurrent()).toHaveValue('');
    expect(passwordNew()).toHaveValue('');
    expect(passwordNewConfirm()).toHaveValue('');
    expect(discardButton()).toHaveClass('disabled');
  });

  it('does not render if no password permission', async () => {
    when(ExtJS.checkPermission)
    .calledWith('astrum:userschangepw:create')
    .mockReturnValue(false);
    const {passwordCurrent, passwordNew, passwordNewConfirm, discardButton} = render();

    expect(passwordCurrent()).not.toBeInTheDocument();
    expect(passwordNew()).not.toBeInTheDocument();
    expect(passwordNewConfirm()).not.toBeInTheDocument();
    expect(discardButton()).not.toBeInTheDocument();
  });

  it('prevents password change when new matches current', async () => {
    let {passwordCurrent, passwordNew, passwordNewConfirm, discardButton} = render();

    await TestUtils.changeField(passwordCurrent, 'foobar');

    await TestUtils.changeField(passwordNew, 'foobar');

    await TestUtils.changeField(passwordNewConfirm, 'foobar');

    userEvent.click(selectors.querySubmitButton());
    expect(selectors.queryFormError(TestUtils.VALIDATION_ERRORS_MESSAGE)).toBeInTheDocument();
    expect(discardButton()).not.toHaveClass('disabled');
  });

  it('prevents password change when new does not match confirm', async () => {
    let {passwordCurrent, passwordNew, passwordNewConfirm, discardButton} = render();

    await TestUtils.changeField(passwordCurrent, 'foobar');

    await TestUtils.changeField(passwordNew, 'bazzle');

    await TestUtils.changeField(passwordNewConfirm, 'bazzl');

    userEvent.click(selectors.querySubmitButton());
    expect(selectors.queryFormError(TestUtils.VALIDATION_ERRORS_MESSAGE)).toBeInTheDocument();
    expect(discardButton()).not.toHaveClass('disabled');
  });

  it('sends the correct password change request', async () => {
    let {passwordCurrent, passwordNew, passwordNewConfirm, changePasswordButton, discardButton} = render();

    await TestUtils.changeField(passwordCurrent, 'foobar');

    await TestUtils.changeField(passwordNew, 'bazzle');

    await TestUtils.changeField(passwordNewConfirm, 'bazzle');

    expect(changePasswordButton()).not.toHaveClass('disabled');
    expect(discardButton()).not.toHaveClass('disabled');

    userEvent.click(changePasswordButton());

    await waitFor(() => expect(Axios.put).toHaveBeenCalledTimes(1));
    expect(Axios.put).toHaveBeenCalledWith(
        CHANGE_PASSWORD_URL,
        {
          authToken: 'fakeToken',
          password: 'bazzle',
        }
    );
    expect(passwordCurrent()).toHaveValue('');
    expect(passwordNew()).toHaveValue('');
    expect(passwordNewConfirm()).toHaveValue('');
    expect(discardButton()).toHaveClass('disabled');
  });

  it('shows password API validation error', async () => {
    const {passwordCurrent, passwordNew, passwordNewConfirm, changePasswordButton} = render();
    const apiValidationError = 'Passwords must contain an uppercase letter'

    await TestUtils.changeField(passwordCurrent, 'foobar');
    await TestUtils.changeField(passwordNew, 'bazzle');
    await TestUtils.changeField(passwordNewConfirm, 'bazzle');

    Axios.put.mockRejectedValue({
      response: {
        data: [
          {
            id: '*',
            message: apiValidationError
          }
        ]
      }
    });

    await act(async () => userEvent.click(changePasswordButton()));

    expect(ExtJS.showErrorMessage).toHaveBeenCalledWith(apiValidationError);
  });

  it('resets the form on discard', async () => {
    let {passwordCurrent, passwordNew, passwordNewConfirm, changePasswordButton, discardButton} = render();

    await TestUtils.changeField(passwordCurrent, 'foobar');

    await TestUtils.changeField(passwordNew, 'bazzle');

    await TestUtils.changeField(passwordNewConfirm, 'bazzle');

    expect(changePasswordButton()).not.toHaveClass('disabled');
    expect(discardButton()).not.toHaveClass('disabled');

    userEvent.click(discardButton());

    await waitFor(() => expect(passwordCurrent()).toHaveValue(''));
    expect(passwordNew()).toHaveValue('');
    expect(passwordNewConfirm()).toHaveValue('');

    expect(discardButton()).toHaveClass('disabled');
  });
});
