
import React from 'react';
import Axios from 'axios';
import {mergeDeepRight} from 'ramda';
import {render, screen, waitForElementToBeRemoved, waitFor} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {act} from 'react-dom/test-utils';
import {when} from 'jest-when';

import {ExtJS, ExtAPIUtils, APIConstants} from '@globalart/astrum-ui-plugin';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import ProprietaryRepositories from './ProprietaryRepositories';
import UIStrings from '../../../../constants/UIStrings';

const {PROPRIETARY_REPOSITORIES: LABELS, SETTINGS} = UIStrings;
const {EXT: {PROPRIETARY_REPOSITORIES: {ACTION, METHODS}, }} = APIConstants;
const URL = APIConstants.EXT.URL;

jest.mock('@globalart/astrum-ui-plugin', () => {
  return {
    ...jest.requireActual('@globalart/astrum-ui-plugin'),
    ExtJS: {
      showSuccessMessage: jest.fn(),
      showErrorMessage: jest.fn(),
      checkPermission: jest.fn().mockReturnValue(true),
    }
  }
});

const selectors = {
  ...TestUtils.selectors,
  ...TestUtils.formSelectors,
  discardButton: () => screen.getByText(SETTINGS.DISCARD_BUTTON_LABEL),
  saveButton: () => screen.getByText(SETTINGS.SAVE_BUTTON_LABEL),
  availableList: () => screen.getByRole('group', {name: LABELS.CONFIGURATION.AVAILABLE_TITLE}),
  selectedList: () => screen.getByRole('group', {name: LABELS.CONFIGURATION.SELECTED_TITLE}),
};

const ENABLED_REPOS_REQUEST = ExtAPIUtils.createRequestBody(ACTION, METHODS.READ, null, 1);
const ALL_REPOS_REQUEST = ExtAPIUtils.createRequestBody(ACTION, METHODS.POSSIBLE_REPOS, null, 2);

const ENABLED_REPOS_RESPONSE = {
  ...TestUtils.makeExtResult({enabledRepositories: ['nuget-hosted']}),
  tid: 1,
}
const ALL_REPOS_RESPONSE = {
  ...TestUtils.makeExtResult([
    {id: "maven-releases", name: "maven-releases"},
    {id: "maven-snapshots", name: "maven-snapshots"},
    {id: "nuget-hosted", name: "nuget-hosted"},
  ]),
  tid: 2,
};

const FETCH_DATA_REQUEST = [ENABLED_REPOS_REQUEST, ALL_REPOS_REQUEST];
const FETCH_DATA_RESPONSE = [ENABLED_REPOS_RESPONSE, ALL_REPOS_RESPONSE];

describe('ProprietaryRepositories', () => {

  beforeEach(() => {
    when(Axios.post).calledWith(URL, FETCH_DATA_REQUEST).mockResolvedValue({
      data: FETCH_DATA_RESPONSE
    });
  });

  it('renders the resolved data', async () => {
    const {queryLoadingMask, availableList, selectedList} = selectors;

    render(<ProprietaryRepositories/>);
    await waitForElementToBeRemoved(queryLoadingMask());

    expect(availableList()).toHaveTextContent('maven-releases');
    expect(selectedList()).not.toHaveTextContent('maven-releases');
    expect(availableList()).toHaveTextContent('maven-snapshots');
    expect(selectedList()).toHaveTextContent('nuget-hosted');
  });

  it('discards changes', async () => {
    const {availableList, selectedList, discardButton} = selectors;

    render(<ProprietaryRepositories/>);
    await waitFor(() => expect(Axios.post).toHaveBeenCalledTimes(1));

    expect(availableList()).toHaveTextContent('maven-snapshots');
    expect(availableList()).toHaveTextContent('maven-releases');
    expect(selectedList()).toHaveTextContent('nuget-hosted');

    expect(discardButton()).toHaveClass('disabled');

    userEvent.click(screen.getByText('maven-snapshots'));
    userEvent.click(screen.getByText('nuget-hosted'));

    expect(selectedList()).toHaveTextContent('maven-snapshots');
    expect(availableList()).toHaveTextContent('maven-releases');
    expect(availableList()).toHaveTextContent('nuget-hosted');

    expect(discardButton()).not.toHaveClass('disabled');
    userEvent.click(discardButton());

    expect(availableList()).toHaveTextContent('maven-snapshots');
    expect(availableList()).toHaveTextContent('maven-releases');
    expect(selectedList()).toHaveTextContent('nuget-hosted');
  });

  it('edits the Proprietary Repositories Form', async () => {
    const {availableList, selectedList, saveButton, queryFormError} = selectors;

    when(Axios.post).calledWith(URL, expect.objectContaining({method: 'update'}))
      .mockResolvedValue({
        data: TestUtils.makeExtResult({enabledRepositories: ['nuget-hosted']})
      });

    await act(async () => {
      render(<ProprietaryRepositories/>);
    });

    await waitFor(() => expect(Axios.post).toHaveBeenCalledTimes(1));

    userEvent.click(screen.getByText('maven-releases'));
    userEvent.click(screen.getByText('nuget-hosted'));

    expect(availableList()).toHaveTextContent('nuget-hosted');
    expect(availableList()).toHaveTextContent('maven-snapshots');
    expect(selectedList()).toHaveTextContent('maven-releases');

    expect(saveButton()).not.toHaveClass('disabled');

    await act(async () => {userEvent.click(saveButton())});

    expect(Axios.post).toHaveBeenCalledWith(
        URL,
        ExtAPIUtils.createRequestBody(ACTION, METHODS.UPDATE, {data: [{enabledRepositories: ['maven-releases']}]})
    );
  });

  describe('Read Only Mode', () => {
    const listItemClass = 'nx-list__text';

    it('shows Proprietary Repositories configuration in Read Only mode', async () => {
      ExtJS.checkPermission.mockReturnValueOnce(false);

      await act(async () => {
        render(<ProprietaryRepositories/>);
      });

      await waitFor(() => expect(Axios.post).toHaveBeenCalledTimes(1));

      expect(screen.getByText(LABELS.CONFIGURATION.SELECTED_TITLE)).toBeInTheDocument();

      expect(screen.getByText('nuget-hosted')).toHaveClass(listItemClass);
    });

    it('Shows empty Proprietary Repositories page in Read Only mode', async () => {
      ExtJS.checkPermission.mockReturnValueOnce(false);

      when(Axios.post).calledWith(URL, FETCH_DATA_REQUEST).mockResolvedValue({
        data: [
            mergeDeepRight(ENABLED_REPOS_RESPONSE, {result: {data: {enabledRepositories: []}}}),
            ALL_REPOS_RESPONSE
        ],
      });

      await act(async () => {
        render(<ProprietaryRepositories/>);
      });
      await waitFor(() => expect(Axios.post).toHaveBeenCalledTimes(1));

      expect(screen.getByText(LABELS.CONFIGURATION.SELECTED_TITLE)).toBeInTheDocument();
      expect(screen.getByText(LABELS.CONFIGURATION.EMPTY_LIST)).toBeInTheDocument();
    });
  })
});
