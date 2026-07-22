
import React from 'react';
import {
  render,
  waitForElementToBeRemoved,
  screen,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import axios from 'axios';
import {values, pick, ascend, descend, sort, prop} from 'ramda';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';
import {when} from 'jest-when';
import {APIConstants, ExtJS} from '@globalart/astrum-ui-plugin';
import {nodes, singleStatus} from './MetricHealth.testdata';

import {convert} from './MetricHealthMachine';
import MetricHealthDetails from './MetricHealthDetails';

jest.mock('axios', () => ({
  ...jest.requireActual('axios'),
  get: jest.fn(),
}));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    state: jest.fn().mockReturnValue({
      getValue: jest.fn(),
    }),
  },
}));

import UIStrings from '../../../../constants/UIStrings';

const {METRIC_HEALTH: LABELS} = UIStrings;
const {
  SORT_DIRECTIONS: {DESC, ASC},
} = APIConstants;

const NODE_DATA = nodes[0];

const sortNodeProps = (field, order = ASC) =>
  sort(
    (order === ASC ? ascend : descend)(prop(field)),
    convert(NODE_DATA.results)
  );

describe('MetricHealthDetails', () => {
  const selectors = {
    ...TestUtils.selectors,
    ...TestUtils.tableSelectors,
    emptyMessage: () => screen.queryByText(LABELS.EMPTY_NODE),
    backButton: () => screen.queryByText(LABELS.BACK_BUTTON),
  };
  const columns = [
    '',
    LABELS.NAME_HEADER,
    LABELS.MESSAGE_HEADER,
    LABELS.ERROR_HEADER,
  ];
  const fields = {
    NAME: 'name',
    MESSAGE: 'message',
  };

  const renderView = async () => {
    const result = render(<MetricHealthDetails />);
    await waitForElementToBeRemoved(TestUtils.selectors.queryLoadingMask());
    return result;
  };

  const expectTableRows = (data, keys) => {
    const rows = TestUtils.tableSelectors.rows();

    expect(rows).toHaveLength(data.length);

    data.forEach((item, index) => {
      const row = rows[index];

      keys.forEach((key) => expect(item.hasOwnProperty(key)).toBeTruthy());

      values(pick(keys, item)).forEach((value, index) => {
        expect(row.cells[index + 1]).toHaveTextContent(value || '');
      });
    });
  };

  describe('HA', () => {
    beforeEach(() => {
      when(ExtJS.state().getValue)
        .calledWith('astrum.datastore.clustered.enabled')
        .mockReturnValue(true);
    });

    it('renders the resolved data', async () => {
      axios.get.mockReturnValue(
        Promise.resolve({
          data: NODE_DATA,
        })
      );

      await renderView();

      TestUtils.expectTableHeaders(columns);
      expect(selectors.backButton()).toBeInTheDocument();
      expect(screen.queryByText(NODE_DATA.hostname)).toBeInTheDocument();
      expectTableRows(convert(NODE_DATA.results), Object.values(fields));
    });

    it('renders message if there is an error', async () => {
      const errorMessage = 'Error';
      axios.get.mockRejectedValue({message: errorMessage});

      await renderView();

      expect(screen.queryByText(errorMessage)).toBeInTheDocument();
    });

    it('renders the resolved empty data', async () => {
      axios.get.mockReturnValue(
        Promise.resolve({
          data: [],
        })
      );

      await renderView();

      expect(selectors.emptyMessage()).toBeInTheDocument();
    });

    describe('Sorting', () => {
      const expectProperRowsOrder = (
        data,
        fieldName = 'name',
        columnIndex = 0
      ) => {
        const rows = TestUtils.tableSelectors.rows();
        data.forEach((item, index) => {
          expect(rows[index].cells[columnIndex + 1]).toHaveTextContent(
            item[fieldName]
          );
        });
      };

      const expectProperOrder = async (fieldName, columnName, direction) => {
        const {headerCell} = selectors;

        const sortedNodeProps = sortNodeProps(fieldName, direction);
        userEvent.click(headerCell(columnName));

        expectProperRowsOrder(sortedNodeProps);
      };

      it('sorts the rows by each columns', async () => {
        axios.get.mockReturnValue(
          Promise.resolve({
            data: NODE_DATA,
          })
        );

        await renderView();

        await expectProperOrder(fields.NAME, LABELS.NAME_HEADER, DESC);
        await expectProperOrder(fields.NAME, LABELS.NAME_HEADER, ASC);

        await expectProperOrder(fields.ERROR, LABELS.ERROR_HEADER, ASC);
        await expectProperOrder(fields.ERROR, LABELS.ERROR_HEADER, DESC);
      });
    });
  });

  describe('Not HA', () => {
    beforeEach(() => {
      when(ExtJS.state().getValue)
        .calledWith('astrum.datastore.clustered.enabled')
        .mockReturnValue(false);
    });

    it('renders the resolved data', async () => {
      axios.get.mockReturnValue(
        Promise.resolve({
          data: singleStatus,
        })
      );

      await renderView();

      TestUtils.expectTableHeaders(columns);
      expect(selectors.backButton()).not.toBeInTheDocument();
      expect(screen.queryByText(NODE_DATA.hostname)).not.toBeInTheDocument();
      expectTableRows(convert(NODE_DATA.results), Object.values(fields));
    });

    it('renders message if there is an error', async () => {
      const errorMessage = 'Error';
      axios.get.mockRejectedValue({message: errorMessage});

      await renderView();

      expect(screen.queryByText(errorMessage)).toBeInTheDocument();
    });

    it('renders the resolved empty data', async () => {
      axios.get.mockReturnValue(
        Promise.resolve({
          data: [],
        })
      );

      await renderView();

      expect(selectors.emptyMessage()).toBeInTheDocument();
    });

    describe('Sorting', () => {
      const expectProperRowsOrder = (
        data,
        fieldName = 'name',
        columnIndex = 0
      ) => {
        const rows = TestUtils.tableSelectors.rows();
        data.forEach((item, index) => {
          expect(rows[index].cells[columnIndex + 1]).toHaveTextContent(
            item[fieldName]
          );
        });
      };

      const expectProperOrder = async (fieldName, columnName, direction) => {
        const {headerCell} = selectors;

        const sortedNodeProps = sortNodeProps(fieldName, direction);
        userEvent.click(headerCell(columnName));

        expectProperRowsOrder(sortedNodeProps);
      };

      it('sorts the rows by each columns', async () => {
        axios.get.mockReturnValue(
          Promise.resolve({
            data: singleStatus,
          })
        );

        await renderView();

        await expectProperOrder(fields.NAME, LABELS.NAME_HEADER, DESC);
        await expectProperOrder(fields.NAME, LABELS.NAME_HEADER, ASC);

        await expectProperOrder(fields.ERROR, LABELS.ERROR_HEADER, ASC);
        await expectProperOrder(fields.ERROR, LABELS.ERROR_HEADER, DESC);
      });
    });
  });
});
