 
import React from 'react';
import axios from 'axios';
import {waitFor, waitForElementToBeRemoved} from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import {ExtJS, APIConstants} from '@globalart/astrum-ui-plugin';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import OAuth2Configuration from './OAuth2Configuration';
import UIStrings from '../../../../constants/UIStrings';

const OAUTH2_API_URL = APIConstants.REST.INTERNAL.OAUTH2;

jest.mock('axios', () => ({
  get: jest.fn(),
  put: jest.fn()
}));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    showSuccessMessage: jest.fn(),
    showErrorMessage: jest.fn()
  }
}));

const DEFAULT_OAUTH2_CONFIGURATION = {
  clientId: "",
  clientSecret: "",
  idpAuthorizationUrl: "",
  idpLogoutUrl: "",
  idpTokenUrl: "",
  idpJwksUrl: "",
  usernameClaim: "",
  firstNameClaim: "",
  lastNameClaim: "",
  emailClaim: "",
  groupsClaim: "",
  idpJwsAlgorithm: "RS256",
  exactMatchClaims: {},
  authorizationCustomParams: {},
  tokenRequestCustomParams: {}
};

const PENDING_REQUEST = () => new Promise(jest.fn());
const DEFAULT_RESPONSE = {data: DEFAULT_OAUTH2_CONFIGURATION};
const ERROR_RESPONSE = {response: {data: [{"id": "FIELD clientId", "message": "must not be empty"}]}};

const selectors = {
  ...TestUtils.selectors,
  ...TestUtils.formSelectors
};

describe('OAuth2Configuration', () => {
  beforeEach(() => {
    window.dirty = [];
  });

  afterEach(() => {
    window.dirty = [];
  });

  function render() {
    const FIELD_LABELS = UIStrings.OAUTH2_CONFIGURATION.FIELDS;

    return TestUtils.render(<OAuth2Configuration/>, ({getByLabelText, getByRole, getByText}) => ({
      errorMessage: () => getByRole('alert'),
      clientIdField: () => getByLabelText(FIELD_LABELS.CLIENT_ID),
      clientSecretField: () => getByLabelText(FIELD_LABELS.CLIENT_SECRET),
      idpAuthorizationUrlField: () => getByLabelText(FIELD_LABELS.IDP_AUTHORIZATION_URL),
      idpLogoutUrlField: () => getByLabelText(FIELD_LABELS.IDP_LOGOUT_URL),
      idpTokenUrlField: () => getByLabelText(FIELD_LABELS.IDP_TOKEN_URL),
      idpJwksUrlField: () => getByLabelText(FIELD_LABELS.IDP_JWKS_URL),
      usernameClaimField: () => getByLabelText(FIELD_LABELS.USERNAME_CLAIM),
      firstNameClaimField: () => getByLabelText(FIELD_LABELS.FIRST_NAME_CLAIM),
      lastNameClaimField: () => getByLabelText(FIELD_LABELS.LAST_NAME_CLAIM),
      emailClaimField: () => getByLabelText(FIELD_LABELS.EMAIL_CLAIM),
      groupsClaimField: () => getByLabelText(FIELD_LABELS.GROUPS_CLAIM),
      idpJwsAlgorithmField: () => getByLabelText(FIELD_LABELS.IDP_JWS_ALGORITHM),
      idpJwksField: () => getByLabelText(FIELD_LABELS.IDP_JWKS),
      authorizationCustomParamsField: () => getByLabelText(FIELD_LABELS.AUTHORIZATION_CUSTOM_PARAMS),
      tokenRequestCustomParamsField: () => getByLabelText(FIELD_LABELS.TOKEN_REQUEST_CUSTOM_PARAMS),
      exactMatchClaimsField: () => getByLabelText(FIELD_LABELS.EXACT_MATCH_CLAIMS),
      discardButton: () => getByText(UIStrings.SETTINGS.DISCARD_BUTTON_LABEL),
      validationError: (error) => getByText(error)
    }));
  }

  it('renders the form with default settings', async () => {
    axios.get.mockResolvedValue(DEFAULT_RESPONSE);

    const {loadingMask, discardButton} = render();

    await waitForElementToBeRemoved(loadingMask);

    expect(discardButton()).toHaveClass('disabled');
  });

  it('shows load error if the load failed', async () => {
    const message = 'Server Error';
    axios.get.mockRejectedValue({message});

    render();

    await waitForElementToBeRemoved(selectors.queryLoadingMask());

    expect(selectors.queryLoadError()).toBeInTheDocument();
  });

  it('does not save when the save button is disabled', async () => {
    axios.get.mockResolvedValue(DEFAULT_RESPONSE);

    render();

    await waitForElementToBeRemoved(selectors.queryLoadingMask());

    userEvent.click(selectors.querySubmitButton());
    expect(selectors.queryFormError(TestUtils.NO_CHANGES_MESSAGE)).toBeInTheDocument();
  });

  it('does not save when the form is pristine', async () => {
    axios.get.mockResolvedValue({
      data: {
        ...DEFAULT_OAUTH2_CONFIGURATION,
        clientId: 'test-client-id',
        clientSecret: 'test-client-secret'
      }
    });

    render();

    await waitForElementToBeRemoved(selectors.queryLoadingMask());

    userEvent.click(selectors.querySubmitButton());
    expect(selectors.queryFormError(TestUtils.NO_CHANGES_MESSAGE)).toBeInTheDocument();
  });

  it('enables the discard button when there are changes and executes it', async () => {
    axios.get.mockResolvedValue(DEFAULT_RESPONSE);

    const {
      loadingMask,
      clientIdField,
      clientSecretField,
      idpAuthorizationUrlField,
      idpLogoutUrlField,
      idpTokenUrlField,
      idpJwksUrlField,
      usernameClaimField,
      firstNameClaimField,
      lastNameClaimField,
      emailClaimField,
      groupsClaimField,
      idpJwsAlgorithmField,
      idpJwksField,
      authorizationCustomParamsField,
      tokenRequestCustomParamsField,
      exactMatchClaimsField,
      discardButton
    } = render();
    
    await waitForElementToBeRemoved(loadingMask);

    await TestUtils.changeField(clientIdField, 'aclientid');
    await TestUtils.changeField(clientSecretField, 'aclientsecret');
    await TestUtils.changeField(idpAuthorizationUrlField, 'http://example.com');
    await TestUtils.changeField(idpLogoutUrlField, 'http://example.com');
    await TestUtils.changeField(idpTokenUrlField, 'http://example.com');
    await TestUtils.changeField(idpJwksUrlField, 'http://example.com');
    await TestUtils.changeField(usernameClaimField, 'username');
    await TestUtils.changeField(firstNameClaimField, 'firstName');
    await TestUtils.changeField(lastNameClaimField, 'lastName');
    await TestUtils.changeField(emailClaimField, 'email');
    await TestUtils.changeField(groupsClaimField, 'groups');
    await TestUtils.changeField(idpJwsAlgorithmField, 'HS256');
    await TestUtils.changeField(idpJwksField, '{"keys": []}');
    await TestUtils.changeField(authorizationCustomParamsField, '{"param":"value"}');
    await TestUtils.changeField(tokenRequestCustomParamsField, '{"param":"value"}');
    await TestUtils.changeField(exactMatchClaimsField, '{"test":"claim"}');

    expect(discardButton()).not.toHaveClass('disabled');
    userEvent.click(discardButton());
    await waitFor(() => expect(discardButton()).toHaveClass('disabled'));

    expect(clientIdField()).toHaveValue('');
    expect(clientSecretField()).toHaveValue('');
    expect(idpAuthorizationUrlField()).toHaveValue('');
    expect(idpLogoutUrlField()).toHaveValue('');
    expect(idpTokenUrlField()).toHaveValue('');
    expect(idpJwksUrlField()).toHaveValue('');
    expect(usernameClaimField()).toHaveValue('');
    expect(firstNameClaimField()).toHaveValue('');
    expect(lastNameClaimField()).toHaveValue('');
    expect(emailClaimField()).toHaveValue('');
    expect(groupsClaimField()).toHaveValue('');
    expect(idpJwsAlgorithmField()).toHaveValue('RS256');
    expect(idpJwksField()).toHaveValue('');
    expect(authorizationCustomParamsField()).toHaveValue('{}');
    expect(tokenRequestCustomParamsField()).toHaveValue('{}');
    expect(exactMatchClaimsField()).toHaveValue('{}');
  });

  it('tells the user the required fields are needed', async () => {
    axios.get.mockResolvedValue(DEFAULT_RESPONSE);

    const {loadingMask, clientIdField} = render();

    await waitForElementToBeRemoved(loadingMask);

    await TestUtils.changeField(clientIdField, '');
    userEvent.click(selectors.querySubmitButton());
    expect(selectors.queryFormError(TestUtils.VALIDATION_ERRORS_MESSAGE));
  });

  it('enables the save button when there are valid changes', async () => {
    axios.get.mockResolvedValue(DEFAULT_RESPONSE);

    const {
      loadingMask,
      clientIdField,
      clientSecretField,
      idpAuthorizationUrlField,
      usernameClaimField
    } = render();

    await waitForElementToBeRemoved(loadingMask);

    await TestUtils.changeField(clientIdField, 'test-client-id');
    await TestUtils.changeField(clientSecretField, 'test-client-secret');
    await TestUtils.changeField(idpAuthorizationUrlField, 'https://idp.example.com/auth');
    await TestUtils.changeField(usernameClaimField, 'sub');
    expect(selectors.queryFormError()).not.toBeInTheDocument();

    await TestUtils.changeField(clientIdField, '');
    userEvent.click(selectors.querySubmitButton());
    expect(selectors.queryFormError(TestUtils.VALIDATION_ERRORS_MESSAGE)).toBeInTheDocument();

    await TestUtils.changeField(usernameClaimField, '');
    userEvent.click(selectors.querySubmitButton());
    expect(selectors.queryFormError(TestUtils.VALIDATION_ERRORS_MESSAGE)).toBeInTheDocument();
  });

  it('saves changes successfully', async () => {
    axios.get.mockResolvedValueOnce(DEFAULT_RESPONSE)

    const {
      loadingMask,
      clientIdField,
      clientSecretField,
      idpAuthorizationUrlField,
      idpLogoutUrlField,
      idpTokenUrlField,
      idpJwksUrlField,
      usernameClaimField,
      firstNameClaimField,
      lastNameClaimField,
      emailClaimField,
      groupsClaimField,
    } = render();

    await waitForElementToBeRemoved(loadingMask);

    await TestUtils.changeField(clientIdField, 'test-client-id');
    await TestUtils.changeField(clientSecretField, 'test-client-secret');
    await TestUtils.changeField(idpAuthorizationUrlField, 'https://idp.example.com/auth');
    await TestUtils.changeField(idpLogoutUrlField, 'https://idp.example.com/logout');
    await TestUtils.changeField(idpTokenUrlField, 'https://idp.example.com/token');
    await TestUtils.changeField(idpJwksUrlField, 'https://idp.example.com/jwks');
    await TestUtils.changeField(usernameClaimField, 'sub');
    await TestUtils.changeField(firstNameClaimField, 'given_name');
    await TestUtils.changeField(lastNameClaimField, 'family_name');
    await TestUtils.changeField(emailClaimField, 'email');
    await TestUtils.changeField(groupsClaimField, 'groups');

    const updatedConfiguration = {
      clientId: 'test-client-id',
      clientSecret: 'test-client-secret',
      idpAuthorizationUrl: 'https://idp.example.com/auth',
      idpLogoutUrl: 'https://idp.example.com/logout',
      idpTokenUrl: 'https://idp.example.com/token',
      idpJwksUrl: 'https://idp.example.com/jwks',
      usernameClaim: 'sub',
      firstNameClaim: 'given_name',
      lastNameClaim: 'family_name',
      emailClaim: 'email',
      groupsClaim: 'groups',
      idpJwsAlgorithm: 'RS256',
      exactMatchClaims: {},
      authorizationCustomParams: {},
      tokenRequestCustomParams: {}
    };

    axios.get.mockResolvedValue({data: updatedConfiguration});
    axios.put.mockResolvedValue({data: updatedConfiguration});

    userEvent.click(selectors.querySubmitButton());
    await waitForElementToBeRemoved(selectors.querySavingMask());

    expect(axios.put).toBeCalledWith(OAUTH2_API_URL, updatedConfiguration);
    expect(ExtJS.showSuccessMessage).toHaveBeenCalledWith(UIStrings.OAUTH2_CONFIGURATION.MESSAGES.SAVE_SUCCESS);
    expect(clientIdField()).toHaveValue(updatedConfiguration.clientId);
    expect(clientSecretField()).toHaveValue(updatedConfiguration.clientSecret);
    expect(idpAuthorizationUrlField()).toHaveValue(updatedConfiguration.idpAuthorizationUrl);
    expect(idpLogoutUrlField()).toHaveValue(updatedConfiguration.idpLogoutUrl);
    expect(idpTokenUrlField()).toHaveValue(updatedConfiguration.idpTokenUrl);
    expect(idpJwksUrlField()).toHaveValue(updatedConfiguration.idpJwksUrl);
    expect(usernameClaimField()).toHaveValue(updatedConfiguration.usernameClaim);
    expect(firstNameClaimField()).toHaveValue(updatedConfiguration.firstNameClaim);
    expect(lastNameClaimField()).toHaveValue(updatedConfiguration.lastNameClaim);
    expect(emailClaimField()).toHaveValue(updatedConfiguration.emailClaim);
    expect(groupsClaimField()).toHaveValue(updatedConfiguration.groupsClaim);
  });

  it('shows a save error if the save failed', async () => {
    axios.get.mockResolvedValue(DEFAULT_RESPONSE);
    axios.put.mockRejectedValue(ERROR_RESPONSE);

    const {
      loadingMask,
      clientIdField,
      clientSecretField,
      idpAuthorizationUrlField,
      idpLogoutUrlField,
      idpTokenUrlField,
      idpJwksUrlField,
      usernameClaimField,
      firstNameClaimField,
      lastNameClaimField,
      emailClaimField,
      groupsClaimField,
      validationError
    } = render();

    await waitForElementToBeRemoved(loadingMask);

    await TestUtils.changeField(clientIdField, 'test-client-id');
    await TestUtils.changeField(clientSecretField, 'test-client-secret');
    await TestUtils.changeField(idpAuthorizationUrlField, 'https://idp.example.com/auth');
    await TestUtils.changeField(idpLogoutUrlField, 'https://idp.example.com/logout');
    await TestUtils.changeField(idpTokenUrlField, 'https://idp.example.com/token');
    await TestUtils.changeField(idpJwksUrlField, 'https://idp.example.com/jwks');
    await TestUtils.changeField(usernameClaimField, 'sub');
    await TestUtils.changeField(firstNameClaimField, 'given_name');
    await TestUtils.changeField(lastNameClaimField, 'family_name');
    await TestUtils.changeField(emailClaimField, 'email');
    await TestUtils.changeField(groupsClaimField, 'groups');

    userEvent.click(selectors.querySubmitButton());
    await waitForElementToBeRemoved(selectors.querySavingMask());

    expect(validationError(ERROR_RESPONSE.response.data[0].message)).toBeInTheDocument();
  });

  it('sets the dirty status appropriately', async () => {
    axios.get.mockResolvedValue(DEFAULT_RESPONSE);

    const {
      loadingMask,
      clientIdField
    } = render();

    await waitForElementToBeRemoved(loadingMask);

    expect(window.dirty).toEqual([]);

    await TestUtils.changeField(clientIdField, 'test-client-id');

    expect(window.dirty).toEqual(['OAuth2ConfigurationForm']);
  });

  it('trims claim field values before saving', async () => {
    axios.get.mockResolvedValueOnce(DEFAULT_RESPONSE)

    const {
      loadingMask,
      clientIdField,
      clientSecretField,
      idpAuthorizationUrlField,
      idpLogoutUrlField,
      idpTokenUrlField,
      idpJwksUrlField,
      usernameClaimField,
      firstNameClaimField,
      lastNameClaimField,
      emailClaimField,
      groupsClaimField,
    } = render();

    const conf = {
      clientId: 'test-client-id',
      clientSecret: 'test-client-secret',
      idpAuthorizationUrl: 'https://idp.example.com/auth',
      idpLogoutUrl: 'https://idp.example.com/logout',
      idpTokenUrl: 'https://idp.example.com/token',
      idpJwksUrl: 'https://idp.example.com/jwks',
      usernameClaim: 'sub',
      firstNameClaim: 'given_name',
      lastNameClaim: 'family_name',
      emailClaim: 'email',
      groupsClaim: 'groups',
      idpJwsAlgorithm: 'RS256',
      exactMatchClaims: {},
      authorizationCustomParams: {},
      tokenRequestCustomParams: {}
    };

    await waitForElementToBeRemoved(loadingMask);

    await TestUtils.changeField(clientIdField, conf.clientId);
    await TestUtils.changeField(clientSecretField, conf.clientSecret);
    await TestUtils.changeField(idpAuthorizationUrlField, conf.idpAuthorizationUrl);
    await TestUtils.changeField(idpLogoutUrlField, conf.idpLogoutUrl);
    await TestUtils.changeField(idpTokenUrlField, conf.idpTokenUrl);
    await TestUtils.changeField(idpJwksUrlField, conf.idpJwksUrl);
    await TestUtils.changeField(usernameClaimField, '  ' + conf.usernameClaim + '  ');
    await TestUtils.changeField(firstNameClaimField, '  ' + conf.firstNameClaim + '  ');
    await TestUtils.changeField(lastNameClaimField, '  ' + conf.lastNameClaim + '  ');
    await TestUtils.changeField(emailClaimField, '  ' + conf.emailClaim + '  ');
    await TestUtils.changeField(groupsClaimField, '  ' + conf.groupsClaim + '  ');

    userEvent.click(selectors.querySubmitButton());
    await waitForElementToBeRemoved(selectors.querySavingMask());

    expect(axios.put).toBeCalledWith(OAUTH2_API_URL, conf);
  });
});
