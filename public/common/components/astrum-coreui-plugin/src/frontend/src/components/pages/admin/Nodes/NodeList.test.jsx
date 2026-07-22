
import React from 'react';
import {render, screen, waitForElementToBeRemoved} from '@testing-library/react';
import {APIConstants} from '@globalart/astrum-ui-plugin';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';
import {when} from 'jest-when';
import Axios from 'axios';
import NodeList from './NodeList';
import UIStrings from '../../../../constants/UIStrings';

const {
  NODES: {HELP}
} = UIStrings;

const {
  REST: {
    INTERNAL: {GET_SUPPORT_ZIP_ACTIVE_NODES}
  }
} = APIConstants;

jest.mock('axios', () => ({
  ...jest.requireActual('axios'),
  get: jest.fn()
}));

const selectors = {
  ...TestUtils.selectors,
  nodeHostname: (hostname) => screen.getByText(hostname),
  helpHeader: () => screen.getByText(HELP.TITLE)
};

describe('NodeList', () => {
  const nodes = [
    {
      nodeId: '111',
      hostname: 'cluster-node-1',
      status: 'NOT_CREATED',
      blobRef: null,
      lastUpdated: null
    },
    {
      nodeId: '222',
      hostname: 'cluster-node-2',
      status: 'NOT_CREATED',
      blobRef: null,
      lastUpdated: null
    },
    {
      nodeId: '333',
      hostname: 'cluster-node-3',
      status: 'NOT_CREATED',
      blobRef: null,
      lastUpdated: null
    }
  ];

  const renderAndWaitForLoad = async () => {
    render(<NodeList />);
    await waitForElementToBeRemoved(selectors.queryLoadingMask());
  };

  beforeEach(() => {
    when(Axios.get).calledWith(GET_SUPPORT_ZIP_ACTIVE_NODES).mockResolvedValue({data: nodes});
  });

  it('renders node cards and help', async () => {
    await renderAndWaitForLoad();

    const {helpHeader, nodeHostname} = selectors;

    expect(helpHeader()).toBeInTheDocument();

    nodes.forEach(({hostname}) => {
      expect(nodeHostname(hostname)).toBeInTheDocument();
    });
  });
});
