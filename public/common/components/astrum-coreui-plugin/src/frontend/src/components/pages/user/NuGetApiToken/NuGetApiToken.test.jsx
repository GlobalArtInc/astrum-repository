
import React from 'react';
import Axios from 'axios';
import {act} from 'react-dom/test-utils';
import {waitFor} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import NuGetApiToken from './NuGetApiToken';
import UIStrings from '../../../../constants/UIStrings';
import {when} from "jest-when";

const mockToken = 'fakeToken'
const mockTokenB64 = 'ZmFrZVRva2Vu'

jest.mock('@globalart/astrum-ui-plugin', () => {
  return {
    ...jest.requireActual('@globalart/astrum-ui-plugin'),
    ExtJS: {
      showSuccessMessage: jest.fn(),
      requestAuthenticationToken: jest.fn(() => Promise.resolve(mockToken)),
      absolutePath: jest.fn(() => 'http://localhost:4242/repository/fakeUrl')
    }
  };
});

describe('NuGetApiToken', () => {
  beforeEach(() => {
    when(Axios.get).calledWith(`/service/rest/internal/nuget-api-key?authToken=${mockTokenB64}`).mockResolvedValue({
      data: {
        apiKey: 'testApiKey'
      }
    });
    when(Axios.delete).calledWith(`/service/rest/internal/nuget-api-key?authToken=${mockTokenB64}`).mockResolvedValue({
      data: {
        apiKey: 'newTestApiKey'
      }
    });
  });

  function renderView(view) {
    return TestUtils.render(view, ({queryByText, getByText}) => ({
      accessButton: () => getByText(UIStrings.NUGET_API_KEY.ACCESS.BUTTON),
      resetButton: () => getByText(UIStrings.NUGET_API_KEY.RESET.BUTTON),
      nugetKey: () => queryByText('testApiKey'),
      newNugetKey: () => queryByText('newTestApiKey')
    }));
  }

  it('renders correctly', async () => {
    let {accessButton, nugetKey} = renderView(<NuGetApiToken/>);

    await waitFor(() =>  expect(nugetKey()).not.toBeInTheDocument());

    await act(async () => userEvent.click(accessButton()));

    await waitFor(() =>  expect(nugetKey()).toBeInTheDocument());
  });

  it('uses the get call when the access button is pressed',  async () => {
    let { accessButton, nugetKey } = renderView(<NuGetApiToken/>);

    await act(async () => userEvent.click(accessButton()));

    await waitFor(() =>  expect(nugetKey()).toBeInTheDocument());

    expect(Axios.get).toHaveBeenCalledTimes(1);
    expect(Axios.get).toHaveBeenCalledWith(
        `/service/rest/internal/nuget-api-key?authToken=${mockTokenB64}`
    );
  });

  it('uses the delete call when the reset button is pressed',  async () => {
    let { resetButton } =  renderView(<NuGetApiToken/>);

    await act(async () => userEvent.click(resetButton()));

    expect(Axios.delete).toHaveBeenCalledTimes(1);
    expect(Axios.delete).toHaveBeenCalledWith(
        `/service/rest/internal/nuget-api-key?authToken=${mockTokenB64}`
    );
  });
});
