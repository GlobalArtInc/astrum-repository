
import React from 'react';
import {act, render, screen} from '@testing-library/react';
import {when} from 'jest-when';
import Axios from 'axios';
import userEvent from '@testing-library/user-event';
import TestUtils from '../../../interface/TestUtils';
import APIConstants from '../../../constants/APIConstants';
import FormUtils from '../../../interface/FormUtils';
import UseAstrumTruststore from './UseAstrumTruststore';
import UIStrings from '../../../constants/UIStrings';

const {
  REST: {
    PUBLIC: {
      SSL_CERTIFICATE_DETAILS: sslCertificateDetailsUrl,
      SSL_CERTIFICATES: sslCertificateUrl,
    }
  }
} = APIConstants;

const {
  USE_TRUST_STORE: LABELS,
  PERMISSION_ERROR,
  SSL_CERTIFICATE_DETAILS,
} = UIStrings;

jest.mock('axios', () => {
  return {
    ...jest.requireActual('axios'),
    get: jest.fn(),
  };
});

describe('UseAstrumTruststore', () => {
  const selectors = {
    label: () => screen.getByText(LABELS.LABEL),
    description: () => screen.getByText(LABELS.DESCRIPTION),
    checkbox: () => screen.getByLabelText(LABELS.DESCRIPTION),
    viewCertificate: () =>
      screen.getByText(LABELS.VIEW_CERTIFICATE).closest('button'),
    tooltipMessage: (selector) => selector().closest('span'),
  };

  const permissions = {
    read: 'astrum:ssl-truststore:read',
    update: 'astrum:ssl-truststore:update',
    create: 'astrum:ssl-truststore:create',
  };

  const mock = jest.fn();

  const mockPermission = (permission, value = true) => {
    when(mock).calledWith(permission).mockReturnValueOnce(value);
  };

  const name = 'truststore';
  const host = 'smtp.gmail.com';
  const port = '465';
  const remoteUrl = `https://${host}:${port}`;
  const send = jest.fn();
  const state = {
    context: {
      data: {},
    },
  };
  const notSecureUrl = `http://${host}:${port}`;

  const renderView = (newUrl) => {
    const url = newUrl || remoteUrl;

    return render(
      <UseAstrumTruststore
        remoteUrl={url}
        onChange={FormUtils.handleUpdate(name, send)}
        {...FormUtils.checkboxProps(name, state)}
      />
    );
  };

  const rerenderView = (render) => {
    return render(
      <UseAstrumTruststore
        remoteUrl={remoteUrl}
        onChange={FormUtils.handleUpdate(name, send)}
        {...FormUtils.checkboxProps(name, state)}
      />
    );
  };

  beforeAll(() => {
    when(Axios.get).calledWith(`${sslCertificateDetailsUrl}?host=${host}&port=${port}`).mockResolvedValue({
      data: [{}],
    });
    when(Axios.get).calledWith(sslCertificateUrl).mockResolvedValue({
      data: [{}],
    });
    global.AS = {
      Permissions: {check: mock},
    };
  });

  it('renders correctly', () => {
    mockPermission(permissions.read);

    renderView();

    expect(selectors.label()).toBeInTheDocument();
    expect(selectors.description()).toBeInTheDocument();
    expect(selectors.viewCertificate()).toBeInTheDocument();
  });

  it('users can mark checkbox regardless of SSL truststore permissions', async () => {
    const {checkbox} = selectors;

    // Test with no SSL truststore permissions - checkbox should still be enabled
    mockPermission(permissions.create, false);
    mockPermission(permissions.update, false);

    const {rerender} = renderView();

    expect(checkbox()).not.toBeDisabled();

    // Test with only create permission - checkbox should still be enabled
    mockPermission(permissions.create, true);
    mockPermission(permissions.update, false);

    rerenderView(rerender);

    expect(checkbox()).not.toBeDisabled();

    // Test with only update permission - checkbox should still be enabled
    mockPermission(permissions.create, false);
    mockPermission(permissions.update, true);

    rerenderView(rerender);

    expect(checkbox()).not.toBeDisabled();

    // Test with both permissions - checkbox should still be enabled
    mockPermission(permissions.create, true);
    mockPermission(permissions.update, true);

    rerenderView(rerender);

    expect(checkbox()).not.toBeDisabled();
  });

  it('users can not view certificate if user does not have enough permissions', async () => {
    const {viewCertificate} = selectors;

    mockPermission(permissions.read, false);

    const {rerender} = renderView();

    expect(viewCertificate()).toHaveClass('disabled');
    expect(viewCertificate()).toHaveAttribute('aria-disabled', 'true');

    await TestUtils.expectToSeeTooltipOnHover(viewCertificate(), PERMISSION_ERROR);

    mockPermission(permissions.read);

    rerenderView(rerender);

    expect(selectors.viewCertificate()).not.toBeDisabled();
  });

  it('users can not add certificate if the url is not secure', async () => {
    const {tooltipMessage, viewCertificate, checkbox} = selectors;

    mockPermission(permissions.read);

    renderView(notSecureUrl);

    expect(checkbox()).toBeDisabled();

    await TestUtils.expectToSeeTooltipOnHover(tooltipMessage(checkbox), LABELS.NOT_SECURE_URL);

    expect(viewCertificate()).toHaveClass('disabled');
    expect(viewCertificate()).toHaveAttribute('aria-disabled', 'true');
    expect(viewCertificate()).toHaveAttribute('title', PERMISSION_ERROR);
  });

  it('users can view certificate', async () => {
    const {viewCertificate} = selectors;

    mockPermission(permissions.read);

    renderView();

    expect(viewCertificate()).not.toBeDisabled();

    await act(async () => userEvent.click(viewCertificate()));

    expect(screen.getByText(SSL_CERTIFICATE_DETAILS.TITLE)).toBeInTheDocument();
  });

  it('users can mark as checked the truststore checkbox', () => {
    const {checkbox} = selectors;

    mockPermission(permissions.read);

    renderView();

    expect(checkbox()).not.toBeDisabled();

    userEvent.click(checkbox());

    expect(send).toHaveBeenCalled();
  });

  it('shows the corresponding tooltip error message for non-secure URLs', async () => {
    const {tooltipMessage, checkbox} = selectors;

    mockPermission(permissions.read);

    renderView(notSecureUrl);

    await TestUtils.expectToSeeTooltipOnHover(tooltipMessage(checkbox), LABELS.NOT_SECURE_URL);
  });
});
