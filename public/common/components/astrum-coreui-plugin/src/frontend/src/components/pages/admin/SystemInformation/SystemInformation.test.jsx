
import React from 'react';
import axios from 'axios';

import {render, screen, waitForElementToBeRemoved, within} from '@testing-library/react';
import {when} from 'jest-when';

import SystemInformation from './SystemInformation';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';
import {ExtJS, APIConstants} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../../constants/UIStrings';
import {
  singleNodeResponse1,
  singleNodeResponse2,
  multiNodeResponse
} from './SystemInformation.testdata';

const {SYSTEM_INFORMATION_HA, SYSTEM_INFORMATION} = APIConstants.REST;

const {ACTIONS} = UIStrings.SYSTEM_INFORMATION;

const NESTED_SECTION = 'h3';
const NAME = '.as-list__term';
const VALUE = '.as-list__description';

jest.mock('axios', () => ({
  ...jest.requireActual('axios'),
  get: jest.fn()
}));

const selectors = {
  ...TestUtils.selectors,
  nodeSelector: () => screen.getByLabelText(ACTIONS.nodeSelector)
};

describe('SystemInformation', () => {
  beforeEach(() => {
    jest.spyOn(ExtJS, 'state').mockReturnValue({getValue: () => false});
  });

  it('renders correctly', async () => {
    when(axios.get).calledWith(SYSTEM_INFORMATION).mockResolvedValue({
      data: singleNodeResponse1
    });

    render(<SystemInformation />);

    await waitForElementToBeRemoved(selectors.queryLoadingMask());

    for (let text in singleNodeResponse1) {
      expectSectionToMatch(singleNodeResponse1, text);
    }
  });
});

describe('SystemInformation HA', () => {
  beforeEach(() => {
    jest.spyOn(ExtJS, 'state').mockReturnValue({getValue: () => true});
  });

  it('renders correctly', async () => {
    when(axios.get).calledWith(SYSTEM_INFORMATION).mockResolvedValue({
      data: singleNodeResponse1
    });
    when(axios.get).calledWith(SYSTEM_INFORMATION_HA).mockResolvedValue({
      data: multiNodeResponse
    });

    render(<SystemInformation />);

    await waitForElementToBeRemoved(selectors.queryLoadingMask());

    expect(selectors.nodeSelector()).toHaveValue('nodeId1');

    for (let text in singleNodeResponse1) {
      expectSectionToMatch(singleNodeResponse1, text);
    }

    await TestUtils.changeField(selectors.nodeSelector, 'nodeId2');

    for (let text in singleNodeResponse2) {
      expectSectionToMatch(singleNodeResponse2, text);
    }
  });
});

function expectSectionToMatch(data, text) {
  const sectionTitle = screen.getByText(text);
  const section = within(sectionTitle.closest('.as-tile-content'));
  expect(sectionTitle).toBeInTheDocument();

  for (let key in data[text]) {
    const isNested = typeof data[text][key] === 'object';
    if (isNested) {
      expect(section.getByText(key, {selector: NESTED_SECTION})).toBeInTheDocument();
      for (let nestedKey in data[text][key]) {
        expect(section.getByText(nestedKey, {selector: NAME})).toBeInTheDocument();
        expect(
          section.getByText(data[text][key][nestedKey], {selector: VALUE})
        ).toBeInTheDocument();
      }
    } else {
      expect(section.getByText(key, {selector: NAME})).toBeInTheDocument();
      expect(section.getByText(data[text][key], {selector: VALUE})).toBeInTheDocument();
    }
  }
}
