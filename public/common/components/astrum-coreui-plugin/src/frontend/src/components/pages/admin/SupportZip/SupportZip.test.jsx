
import React from 'react';
import {render, screen, act, waitFor, within} from '@testing-library/react';
import {when} from 'jest-when';
import userEvent from '@testing-library/user-event';
import {ExtJS, APIConstants} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../../constants/UIStrings';
import Axios from 'axios';
import SupportZip from './SupportZip';

const {SUPPORT_ZIP: LABELS} = UIStrings;

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    state: jest.fn().mockReturnValue({
      getValue: jest.fn(),
      isClustered: jest.fn(),
    }),
  },
}));

describe('SupportZip', () => {
  const selectors = {
    generateAllSupportZipButton: () =>
      screen.getByText(LABELS.GENERATE_ALL_ZIP_FILES),
    modal: () => screen.getByRole('dialog'),
    modalCreateZipButton: () =>
      within(selectors.modal()).getAllByRole('button')[1],
  };

  beforeEach(() => {
    when(ExtJS.state().getValue)
      .calledWith('astrum.datastore.clustered.enabled')
      .mockReturnValue(false);
  });

  const renderView = () => render(<SupportZip />);

  it('renders support zip form', function () {
    renderView();

    const checkboxes = screen.getAllByRole('checkbox');

    expect(checkboxes).toHaveLength(12);

    checkboxes.forEach((checkbox) => expect(checkbox).toBeChecked());
  });

  it('updates the parameters when clicked', function () {
    renderView();

    const checkboxes = screen.getAllByRole('checkbox');

    checkboxes.forEach((checkbox) => expect(checkbox).toBeChecked());

    checkboxes.forEach((checkbox) => {
      userEvent.click(checkbox);
    });

    checkboxes.forEach((checkbox) => expect(checkbox).not.toBeChecked());
  });

  describe('HA', () => {
    const nodeMock = {
      nodeId: '4fe378d0-fd09-4d5b-9d65-eb976a20a769',
      hostname: 'MacBook-Pro.local',
      status: 'NOT_CREATED',
      blobRef: null,
      lastUpdated: null,
    };

    beforeEach(() => {
      when(ExtJS.state().getValue)
        .calledWith('astrum.datastore.clustered.enabled')
        .mockReturnValue(true);

      when(Axios.delete)
        .calledWith(
          APIConstants.REST.INTERNAL.CLEAR_SUPPORT_ZIP_HISTORY + nodeMock.nodeId
        )
        .mockResolvedValue();

      when(Axios.post)
        .calledWith(
          APIConstants.REST.INTERNAL.CLEAR_SUPPORT_ZIP_HISTORY + nodeMock.nodeId
        )
        .mockResolvedValue({
          data: {},
        });

      const responseActiveNodes = {
        data: [nodeMock],
      };

      const responseBlobStores = {
        data: [{}],
      };

      when(Axios.get)
        .calledWith(APIConstants.REST.INTERNAL.GET_SUPPORT_ZIP_ACTIVE_NODES)
        .mockResolvedValue(responseActiveNodes);

      when(Axios.get)
        .calledWith(APIConstants.REST.PUBLIC.BLOB_STORES)
        .mockResolvedValue(responseBlobStores);

      when(Axios.all)
        .calledWith(expect.anything())
        .mockResolvedValue([responseActiveNodes, responseBlobStores]);
    });

    it('users can generate support zip files for all the available nodes', async () => {
      const {generateAllSupportZipButton, modalCreateZipButton} = selectors;

      renderView();

      await waitFor(async () => await screen);

      expect(generateAllSupportZipButton()).toBeInTheDocument();

      await act(async () => userEvent.click(generateAllSupportZipButton()));

      await act(async () => userEvent.click(modalCreateZipButton()));

      expect(Axios.delete).toHaveBeenCalledWith(
        APIConstants.REST.INTERNAL.CLEAR_SUPPORT_ZIP_HISTORY + nodeMock.nodeId
      );

      await waitFor(async () => await screen);

      expect(Axios.post).toHaveBeenCalledWith(
        APIConstants.REST.INTERNAL.SUPPORT_ZIP + nodeMock.nodeId,
        {
          archivedLog: 0,
          auditLog: true,
          configuration: true,
          hostname: 'MacBook-Pro.local',
          jmx: true,
          limitFileSizes: true,
          limitZipSize: true,
          log: true,
          metrics: true,
          replication: true,
          security: true,
          systemInformation: true,
          taskLog: true,
          threadDump: true,
        }
      );
    });

    it('users can generate support zip for a single node', async () => {
      const {modalCreateZipButton} = selectors;

      renderView();

      await waitFor(async () => await screen);

      const button = screen.getByText(LABELS.GENERATE_NEW_ZIP_FILE);

      expect(button).toBeInTheDocument();

      await act(async () => userEvent.click(button));

      await act(async () => userEvent.click(modalCreateZipButton()));

      expect(Axios.delete).toHaveBeenCalledWith(
        APIConstants.REST.INTERNAL.CLEAR_SUPPORT_ZIP_HISTORY + nodeMock.nodeId
      );

      expect(Axios.post).toHaveBeenCalledWith(
        APIConstants.REST.INTERNAL.SUPPORT_ZIP + nodeMock.nodeId,
        {
          archivedLog: 0,
          auditLog: true,
          configuration: true,
          hostname: 'MacBook-Pro.local',
          jmx: true,
          limitFileSizes: true,
          limitZipSize: true,
          log: true,
          metrics: true,
          replication: true,
          security: true,
          systemInformation: true,
          taskLog: true,
          threadDump: true,
        }
      );
    });
  });
});
