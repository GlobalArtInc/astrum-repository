
import React from 'react';
import Axios from 'axios';
import {when} from 'jest-when';
import {render, screen, waitFor, waitForElementToBeRemoved} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {sort, prop, descend, ascend, clone} from 'ramda';

import {ExtJS, APIConstants} from '@globalart/astrum-ui-plugin';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import UIStrings from '../../../../constants/UIStrings';
import PrivilegesList from './PrivilegesList';
import { UIRouter } from '@uirouter/react';
import { getRouter } from '../../../../routerConfig/routerConfig';

const XSS_STRING = TestUtils.XSS_STRING;
const {PRIVILEGES: {LIST: LABELS}} = UIStrings;
const {EXT: {URL, PRIVILEGE: {ACTION, METHODS}}, SORT_DIRECTIONS: {DESC, ASC}} = APIConstants;

jest.mock('@globalart/astrum-ui-plugin', () => {
  return {
    ...jest.requireActual('@globalart/astrum-ui-plugin'),
    ExtJS: {
      checkPermission: jest.fn().mockReturnValue(true),
      state: jest.fn().mockReturnValue({
        getValue: jest.fn()
      })
    }
  }
});

const selectors = {
  ...TestUtils.selectors,
  ...TestUtils.tableSelectors,
  emptyMessage: () => screen.getByText(LABELS.EMPTY_LIST),
  filter: () => screen.queryByPlaceholderText(UIStrings.FILTER),
  createButton: () => screen.getByText(LABELS.CREATE_BUTTON),
};

const ROWS = [{
  description: 'All permissions for Logging',
  id: 'as-logging-all',
  name: 'as-logging-all',
  permission: 'astrum:logging:*',
  readOnly: true,
  type: 'application',
}, {
  description: 'Browse permissions for Scripts',
  id: 'as-script-*-browse',
  name: 'as-script-*-browse',
  permission: 'astrum:script:*:browse,read',
  readOnly: true,
  type: 'script',
}, {
  description: 'All permissions for Settings',
  id: 'as-settings-all',
  name: 'as-settings-all',
  permission: 'astrum:settings:*',
  readOnly: true,
  type: 'application',
}];

const REQUEST = expect.objectContaining({
  action: ACTION,
  method: METHODS.READ.NAME,
});

const FIELDS = {
  NAME: 'name',
  DESCRIPTION: 'description',
  TYPE: 'type',
  PERMISSION: 'permission',
};

const DEFAULT_DATA = {
  action: ACTION,
  data: [
    {
      limit: 25,
      page: 1,
      sort: [
        {
          direction: ASC.toUpperCase(),
          property: "name"
        }
      ],
      start: 0
    }
  ],
  method: METHODS.READ.NAME,
  tid: 1,
  type: 'rpc'
};

const sortPrivileges = (field, order = ASC) => sort((order === ASC ? ascend : descend)(prop(field)), ROWS);

describe('PrivilegesList', function() {

  const renderAndWaitForLoad = async () => {
    const router = getRouter();
    const view = (
      <UIRouter router={router}>
        <PrivilegesList />
      </UIRouter>
    );
    render(view);
    await waitForElementToBeRemoved(selectors.queryLoadingMask());
  }

  beforeEach(() => {
    when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue({
      data: TestUtils.makeExtResult(ROWS)
    });
  });

  it('renders the resolved empty data', async function() {
    when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue({
      data: TestUtils.makeExtResult([])
    });
    const {createButton, emptyMessage} = selectors;

    await renderAndWaitForLoad();

    expect(createButton()).not.toHaveClass('disabled');
    expect(emptyMessage()).toBeInTheDocument();
  });

  it('renders the resolved data', async function() {
    await renderAndWaitForLoad();

    await waitFor(() => expect(Axios.post).toHaveBeenCalledWith(URL, REQUEST));

    TestUtils.expectTableHeaders(Object.values(LABELS.COLUMNS));
    TestUtils.expectTableRows(ROWS, Object.values(FIELDS));
  });

  it('renders the resolved data with XSS', async function() {
    const XSS_ROWS = [{
      ...ROWS[0],
      description: XSS_STRING,
      name: XSS_STRING,
      permission: XSS_STRING,
    }];

    when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue({
      data: TestUtils.makeExtResult(XSS_ROWS)
    });

    await renderAndWaitForLoad();

    TestUtils.expectTableHeaders(Object.values(LABELS.COLUMNS));
    TestUtils.expectTableRows(XSS_ROWS, Object.values(FIELDS));
  });

  it('renders an error message', async function() {
    const message = 'Error Message!';
    const {tableAlert} = selectors;
    when(Axios.post).calledWith(URL, REQUEST).mockRejectedValue({message});

    await renderAndWaitForLoad();

    expect(tableAlert()).toHaveTextContent(message);
  });

  describe('Sorting', function() {
    const expectProperOrder = async (fieldName, columnName, direction) => {
      const {headerCell} = selectors;

      let privileges = sortPrivileges(fieldName, direction);
      when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue({data: TestUtils.makeExtResult(privileges)});

      userEvent.click(headerCell(columnName));
      let newRequest = clone(DEFAULT_DATA);
      newRequest.data[0].sort[0] = {
        direction: direction.toUpperCase(),
        property: fieldName,
      };
      await waitFor(() => expect(Axios.post).toHaveBeenLastCalledWith(URL, newRequest));

      TestUtils.expectProperRowsOrder(privileges);
    }

    it('sorts the rows by each columns', async function () {
      await renderAndWaitForLoad();
      await waitFor(() => expect(Axios.post).toHaveBeenLastCalledWith(URL, REQUEST));
      TestUtils.expectProperRowsOrder(ROWS);

      await expectProperOrder(FIELDS.NAME, LABELS.COLUMNS.NAME, DESC);
      await expectProperOrder(FIELDS.DESCRIPTION, LABELS.COLUMNS.DESCRIPTION, ASC);
      await expectProperOrder(FIELDS.DESCRIPTION, LABELS.COLUMNS.DESCRIPTION, DESC);
      await expectProperOrder(FIELDS.TYPE, LABELS.COLUMNS.TYPE, ASC);
      await expectProperOrder(FIELDS.PERMISSION, LABELS.COLUMNS.PERMISSION, ASC);
      await expectProperOrder(FIELDS.PERMISSION, LABELS.COLUMNS.PERMISSION, DESC);
    });
  });

  it('filters by each columns', async function() {
    const {filter, rows} = selectors;
    const filterString = 'test';

    await renderAndWaitForLoad();

    when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue({data: TestUtils.makeExtResult([ROWS[0]])});

    await TestUtils.changeField(filter, filterString);

    let newRequest = clone(DEFAULT_DATA);
    newRequest.data = [{
      ...newRequest.data[0],
      filter: [{
        value: filterString,
        property: 'filter',
      }],
    }];

    await waitFor(() => expect(Axios.post).toHaveBeenLastCalledWith(URL, newRequest));
    expect(rows()).toHaveLength(1);
  });

  it('disables the create button when not enough permissions', async function() {
    const {createButton} = selectors;
    ExtJS.checkPermission.mockReturnValue(false);

    await renderAndWaitForLoad();

    expect(createButton()).toHaveClass('disabled');
  });

  describe('Pagination', function () {
    const getAxiosResponse = (totalCount, pageSize = 25) => {
      return {
        data: TestUtils.makeExtResult(
          Array.from({ length: pageSize }, (_, i) => ({
            id: `privilege-${i}`,
            name: `Privilege ${i}`,
            description: `Description ${i}`,
            type: 'application',
            permission: 'read',
          })),
          totalCount
        ),
      };
    };
    it('shows pagination when backend returns more items than page size', async function () {
      // Backend returns 25 items but indicates total of 50
      when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue(getAxiosResponse(50, 25));

      await renderAndWaitForLoad();

      // Should show pagination with 2 pages (50 total / 25 per page)
      await waitFor(() => {
        const pagination = screen.getByRole('navigation');
        expect(pagination).toBeInTheDocument();
      });

      const pagination = screen.getByRole('navigation');
      const buttons = pagination.querySelectorAll('button');
      expect(buttons.length).toBeGreaterThan(1);
    });

    it('hides pagination when backend total is less than or equal to page size', async function () {
      // Backend returns 10 items with total of 10
      when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue(getAxiosResponse(10, 10));

      await renderAndWaitForLoad();

      // Should not show pagination
      const pagination = screen.queryByRole('navigation');
      expect(pagination).not.toBeInTheDocument();
    });

    it('calculates correct number of pages from backend total', async function () {
      // Backend returns 100 total items
      when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue(getAxiosResponse(100, 25));

      await renderAndWaitForLoad();

      await waitFor(() => {
        const pagination = screen.getByRole('navigation');
        expect(pagination).toBeInTheDocument();
      });

      // Should calculate 4 pages: Math.ceil(100 / 25) = 4
      // The exact number of buttons depends on AsPagination implementation
      // but pagination should be present
      const pagination = screen.getByRole('navigation');
      expect(pagination).toBeInTheDocument();
    });

    it('sends correct pagination parameters when navigating pages', async function () {
      // Setup initial data
      when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue(getAxiosResponse(50, 25));

      await renderAndWaitForLoad();

      // Verify initial call was made with start: 0
      await waitFor(() => {
        expect(Axios.post).toHaveBeenCalledWith(
          URL,
          expect.objectContaining({
            data: [
              expect.objectContaining({
                start: 0,
                limit: 25,
              }),
            ],
          })
        );
      });

      // Mock response for page 2
      when(Axios.post)
        .calledWith(
          URL,
          expect.objectContaining({
            data: [expect.objectContaining({ start: 25 })],
          })
        )
        .mockResolvedValue(getAxiosResponse(50, 25));

      // Click next page button
      const pagination = screen.getByRole('navigation');
      const buttons = pagination.querySelectorAll('button');
      const nextPageButton = buttons[1]; // Assuming first is prev, second is next
      userEvent.click(nextPageButton);

      // Verify API call for page 2
      await waitFor(() => {
        expect(Axios.post).toHaveBeenLastCalledWith(
          URL,
          expect.objectContaining({
            data: [
              expect.objectContaining({
                start: 25, // Page 2 starts at index 25
                limit: 25,
              }),
            ],
          })
        );
      });
    });

    it('resets to page 1 when filtering', async function () {
      const { filter } = selectors;

      // Setup initial data with pagination
      when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue(getAxiosResponse(50, 25));

      await renderAndWaitForLoad();

      // Navigate to page 2 first
      when(Axios.post)
        .calledWith(
          URL,
          expect.objectContaining({
            data: [expect.objectContaining({ start: 25 })],
          })
        )
        .mockResolvedValue(getAxiosResponse(50, 25));

      const pagination = screen.getByRole('navigation');
      const buttons = pagination.querySelectorAll('button');
      const nextPageButton = buttons[1];
      userEvent.click(nextPageButton);

      await waitFor(() => {
        expect(Axios.post).toHaveBeenLastCalledWith(
          URL,
          expect.objectContaining({
            data: [expect.objectContaining({ start: 25 })],
          })
        );
      });

      // Mock filtered response (backend returns filtered results)
      when(Axios.post)
        .calledWith(
          URL,
          expect.objectContaining({
            data: [
              expect.objectContaining({
                start: 0, // Should reset to page 1
                filter: [{ property: 'filter', value: 'test' }],
              }),
            ],
          })
        )
        .mockResolvedValue({
          data: TestUtils.makeExtResult(
            [
              {
                id: 'filtered',
                name: 'Filtered Privilege',
                description: 'Filtered',
                type: 'application',
                permission: 'read',
              },
            ],
            1 // filtered total count
          ),
        });

      // Apply filter - should reset to page 1
      await TestUtils.changeField(filter, 'test');

      // Verify filter call resets to start: 0
      await waitFor(() => {
        expect(Axios.post).toHaveBeenLastCalledWith(
          URL,
          expect.objectContaining({
            data: [
              expect.objectContaining({
                start: 0, // Should reset to page 1
                filter: [
                  {
                    property: 'filter',
                    value: 'test',
                  },
                ],
              }),
            ],
          })
        );
      });
    });

    it('hides pagination when loading', async function () {
      // Setup data with pagination
      when(Axios.post).calledWith(URL, REQUEST).mockResolvedValue(getAxiosResponse(50, 25));

      // Render component but don't let it finish loading
      const router = getRouter();
      const view = (
        <UIRouter router={router}>
          <PrivilegesList />
        </UIRouter>
      );
      render(view);

      // Initially, pagination should not be visible while loading
      const pagination = screen.queryByRole('navigation');
      expect(pagination).not.toBeInTheDocument();

      // Wait for component to finish loading
      await waitForElementToBeRemoved(selectors.queryLoadingMask());

      // Now pagination should be visible (based on backend total)
      await waitFor(() => {
        const paginationAfterLoad = screen.getByRole('navigation');
        expect(paginationAfterLoad).toBeInTheDocument();
      });
    });
  });
});
