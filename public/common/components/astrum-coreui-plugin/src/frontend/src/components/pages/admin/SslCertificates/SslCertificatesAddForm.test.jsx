
import React from 'react';
import {render, screen, waitFor, within, act} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {when} from 'jest-when';
import Axios from 'axios';

import {ExtJS, APIConstants} from '@globalart/astrum-ui-plugin';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import UIStrings from '../../../../constants/UIStrings';
import SslCertificatesAddForm from './SslCertificatesAddForm';
import {SSL_CERTIFICATES, SSL_CERTIFICATES_MAP} from './SslCertificates.testdata';
import { UIRouter, useCurrentStateAndParams } from '@uirouter/react';
import {getRouter} from "../../../../routerConfig/routerConfig";

const {EXT} = APIConstants;

const {
  SSL_CERTIFICATES: {
    FORM: LABELS,
    ADD_FORM: {
      CAPTION: ADD_FORM_CAPTION,
      LOAD_BUTTON,
      PEM,
      SERVER,
      MODAL: {
        VIEW_BUTTON,
        CONTENT: MODAL_CONTENT
      }
    }
  },
  SETTINGS: {CANCEL_BUTTON_LABEL}
} = UIStrings;

jest.mock('axios', () => ({
  ...jest.requireActual('axios'),
  get: jest.fn(),
  put: jest.fn(),
  post: jest.fn(),
  delete: jest.fn(),
}));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    requestConfirmation: jest.fn(),
    checkPermission: jest.fn(),
    showErrorMessage: jest.fn(),
    showSuccessMessage: jest.fn(),
    state: jest.fn().mockReturnValue({
      getValue: jest.fn()
    })
  },
}));

jest.mock('@uirouter/react', () => ({
  ...jest.requireActual('@uirouter/react'),
    useCurrentStateAndParams: jest.fn(),
}));

const testId = SSL_CERTIFICATES[1].id;
const DATA = SSL_CERTIFICATES_MAP[testId];

const selectors = {
  ...TestUtils.selectors,
  subjectSection: () => screen.getByRole('heading', {name: LABELS.SECTIONS.SUBJECT}).closest('section'),
  subjectCommonName: () => within(selectors.subjectSection()).getByText(LABELS.COMMON_NAME.LABEL).nextSibling,
  subjectOrganization: () => within(selectors.subjectSection()).getByText(LABELS.ORGANIZATION.LABEL).nextSibling,
  subjectOrganizationalUnit: () => within(selectors.subjectSection()).getByText(LABELS.UNIT.LABEL).nextSibling,
  issuerSection: () => screen.getByRole('heading', {name: LABELS.SECTIONS.ISSUER}).closest('section'),
  issuerCommonName: () => within(selectors.issuerSection()).getByText(LABELS.COMMON_NAME.LABEL).nextSibling,
  issuerOrganization: () => within(selectors.issuerSection()).getByText(LABELS.ORGANIZATION.LABEL).nextSibling,
  issuerOrganizationalUnit: () => within(selectors.issuerSection()).getByText(LABELS.UNIT.LABEL).nextSibling,
  certificateSection: () => screen.getByRole('heading', {name: LABELS.SECTIONS.CERTIFICATE}).closest('section'),
  issuedOn: () => within(selectors.certificateSection()).getByText(LABELS.ISSUED_ON.LABEL).nextSibling,
  expiresOn: () => within(selectors.certificateSection()).getByText(LABELS.VALID_UNTIL.LABEL).nextSibling,
  fingerprint: () => within(selectors.certificateSection()).getByText(LABELS.FINGERPRINT.LABEL).nextSibling,
  warning: () => screen.getByText(LABELS.WARNING),
  cancelButton: () => screen.getByText(CANCEL_BUTTON_LABEL),
  deleteButton: () => screen.getByText(LABELS.BUTTONS.DELETE),
  addFormTitle: () => screen.queryByText(ADD_FORM_CAPTION),
  loadFromServerRadioButton: () => screen.getByLabelText(SERVER.RADIO_DESCRIPTION),
  loadFromPemRadioButton: () => screen.getByLabelText(PEM.RADIO_DESCRIPTION),
  remoteHostUrlInput: () => screen.getByLabelText(SERVER.LABEL),
  pemInput: () => screen.getByLabelText(PEM.LABEL),
  loadCertificateButton: () => screen.getByText(LOAD_BUTTON),
  alreadyExistsModal: () => screen.queryByRole('dialog'),
  viewExistingCertificateButton: () => within(selectors.alreadyExistsModal()).getByText(VIEW_BUTTON),
  alreadyExistsModalContent: () => within(selectors.alreadyExistsModal()).getByText(MODAL_CONTENT)
};

describe('SslCertificatesAddForm', function () {
  const renderAddForm = async () => {
    const router = getRouter();
    const result = render(<UIRouter router={router}><SslCertificatesAddForm /></UIRouter>);
    await waitFor(() => expect(selectors.addFormTitle()).toBeInTheDocument());
    return result;
  };

  const loadFromServerExtReqBody = (remoteHostUrl) => ({
    action: EXT.SSL.ACTION,
    method: EXT.SSL.METHODS.RETRIEVE_FROM_HOST,
    data: [remoteHostUrl, null, null],
    type: 'rpc',
    tid: 1
  });

  const loadFromPemExtReqBody = (pem) => ({
    action: EXT.SSL.ACTION,
    method: EXT.SSL.METHODS.DETAILS,
    data: [pem],
    type: 'rpc',
    tid: 1
  });

  const loadFromServerExtErrorRespBody = (errorMsg) => ({
    action: EXT.SSL.ACTION,
    method: EXT.SSL.METHODS.RETRIEVE_FROM_HOST,
    result: {
      message: errorMsg,
      authenticationRequired: false,
      success: false,
      data: []
    },
    type: 'rpc',
    tid: 1
  });

  const loadFromServerExtRespBody = (certificate) => ({
    action: EXT.SSL.ACTION,
    method: EXT.SSL.METHODS.RETRIEVE_FROM_HOST,
    result: {
      success: true,
      data: certificate
    },
    type: 'rpc',
    tid: 1,
  });

  const mockResponse = (response = {}, remoteHost = '') =>{
    when(Axios.post).calledWith(EXT.URL, loadFromServerExtReqBody(remoteHost)).mockResolvedValue(response);
  }

  beforeEach(() => {
    ExtJS.checkPermission.mockReturnValue(true);

    useCurrentStateAndParams.mockReset();
    useCurrentStateAndParams.mockReturnValue({params: {}});
  });

  it('renders correct initial state', async function () {
    await renderAddForm();

    const {
      loadFromServerRadioButton,
      loadFromPemRadioButton,
      remoteHostUrlInput,
      pemInput
    } = selectors;

    expect(loadFromServerRadioButton()).toBeChecked();
    expect(loadFromPemRadioButton()).not.toBeChecked();
    expect(remoteHostUrlInput()).toBeEnabled();
    expect(pemInput()).toBeDisabled();
  });

  it('loads certificate from server', async function () {
    await renderAddForm();

    const {remoteHostUrlInput, loadCertificateButton} = selectors;

    const remoteHost = 'foo.bar';

    await TestUtils.changeField(remoteHostUrlInput, remoteHost);

    await act(async () => userEvent.click(loadCertificateButton()));

    expect(Axios.post).toBeCalledWith(EXT.URL, loadFromServerExtReqBody(remoteHost));
  });

  it('loads certificate from pem', async function () {
    await renderAddForm();

    const {
      pemInput,
      remoteHostUrlInput,
      loadFromPemRadioButton,
      loadCertificateButton
    } = selectors;

    const pemContent = 'CORRECTPEMCONTENT';

    await waitFor(() => expect(pemInput()).toBeDisabled());

    userEvent.click(loadFromPemRadioButton());

    expect(remoteHostUrlInput()).toBeDisabled();
    expect(pemInput()).toBeEnabled();

    await TestUtils.changeField(pemInput, pemContent);

    mockResponse();

    userEvent.click(loadCertificateButton());

    await waitFor(() => {
      expect(Axios.post).toBeCalledWith(EXT.URL, loadFromPemExtReqBody(pemContent));
    });
  });

  it('shows add form with error toast on load from server error', async function () {
    const remoteHost = 'bad.host';
    const errorMessage = 'Could not retrieve an SSL certificate';

    const response = {
      data: loadFromServerExtErrorRespBody(errorMessage)
    }

    mockResponse(response, remoteHost);

    await renderAddForm();

    const {remoteHostUrlInput, loadCertificateButton} = selectors;

    await TestUtils.changeField(remoteHostUrlInput, remoteHost);

    await act(async () => userEvent.click(loadCertificateButton()));

    expect(Axios.post).toBeCalledWith(EXT.URL, loadFromServerExtReqBody(remoteHost));
    expect(remoteHostUrlInput()).toBeVisible();
    expect(screen.getByRole('alert')).toHaveTextContent(errorMessage);
  });

  it('shows modal when trying to add existing certificate', async function () {
    const remoteHost = 'foo.bar';

    const certificate = SSL_CERTIFICATES[3];

    const response = {data: loadFromServerExtRespBody(certificate)};

    when(Axios.post).calledWith(EXT.URL, loadFromServerExtReqBody(remoteHost)).mockResolvedValue(response);

    await renderAddForm();

    const {
      remoteHostUrlInput,
      loadCertificateButton,
      alreadyExistsModal,
      alreadyExistsModalContent,
      viewExistingCertificateButton
    } = selectors;

    await TestUtils.changeField(remoteHostUrlInput, remoteHost);

    userEvent.click(loadCertificateButton());

    await waitFor(() => expect(Axios.post).toBeCalledWith(EXT.URL, loadFromServerExtReqBody(remoteHost)));

    expect(alreadyExistsModal()).toBeVisible();

    expect(alreadyExistsModalContent()).toBeVisible();

    expect(viewExistingCertificateButton()).toBeVisible();
  });
});
