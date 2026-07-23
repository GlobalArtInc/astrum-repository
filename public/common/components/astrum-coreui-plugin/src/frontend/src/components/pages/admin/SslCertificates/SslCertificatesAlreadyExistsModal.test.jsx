
import React from 'react';
import {render, screen, within} from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import {ExtJS} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import SslCertificatesAlreadyExistsModal from './SslCertificatesAlreadyExistsModal';

const {
  SSL_CERTIFICATES: {
    ADD_FORM: {
      MODAL: {HEADER, CONTENT, VIEW_BUTTON}
    }
  },
  SETTINGS: {CANCEL_BUTTON_LABEL}
} = UIStrings;

const stateServiceGoMock = jest.fn();

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    setDirtyStatus: jest.fn(),
    state: jest.fn().mockReturnValue({
      getValue: jest.fn()
    })
  },
}));

jest.mock('@uirouter/react', () => ({
  ...jest.requireActual('@uirouter/react'),
  useRouter: () => ({
    stateService: {
      go: stateServiceGoMock,
    }
  })
}));

const selectors = {
  modal: () => screen.getByRole('dialog'),
  header: () => within(selectors.modal()).getByText(HEADER),
  content: () => within(selectors.modal()).getByText(CONTENT),
  cancelButton: () => within(selectors.modal()).getByText(CANCEL_BUTTON_LABEL),
  viewButton: () => within(selectors.modal()).getByText(VIEW_BUTTON),
};

describe('SslCertificatesAlreadyExistsModal', function () {
  const certificateId = 'test-cert-id';
  const mockCancel = jest.fn();

  const renderModal = () => {
    return render(
      <SslCertificatesAlreadyExistsModal
        certificateId={certificateId}
        cancel={mockCancel}
      />
    );
  };

  it('render data', async function () {
    renderModal();

    expect(selectors.modal()).toBeVisible();
    expect(selectors.header()).toBeInTheDocument();
    expect(selectors.content()).toBeInTheDocument();
  });

  it('click on view button', async function () {
    renderModal();
    userEvent.click(selectors.viewButton());

    expect(ExtJS.setDirtyStatus).toHaveBeenCalledWith('SslCertificatesAddFormMachine', false);
    expect(stateServiceGoMock).toHaveBeenCalledWith(
      'admin.security.sslcertificates.edit',
      {itemId: encodeURIComponent(certificateId)}
    );
  });

  it('click on cancel button', async function () {
    renderModal();
    userEvent.click(selectors.cancelButton());

    expect(ExtJS.setDirtyStatus).toHaveBeenCalledWith('SslCertificatesAddFormMachine', false);
    expect(mockCancel).toHaveBeenCalled();
  });
});
