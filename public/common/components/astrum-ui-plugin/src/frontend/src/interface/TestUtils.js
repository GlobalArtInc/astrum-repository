
import {fireEvent, render, screen, waitFor, within, waitForElementToBeRemoved} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import UIStrings from "../constants/UIStrings";
import {pick} from 'ramda';
import { interpret } from 'xstate';

const {SETTINGS: {SAVE_BUTTON_LABEL, DISCARD_BUTTON_LABEL}} = UIStrings;

/**
 * @since 3.24
 */
export default class TestUtils {
  static REQUIRED_MESSAGE = 'This field is required';
  static VALIDATION_ERRORS_MESSAGE = 'Validation errors are present';
  static NO_CHANGES_MESSAGE = 'There are no changes';
  static THERE_WERE_ERRORS = 'There were validation errors.';
  static SAVE_ERROR = 'An error occurred saving data.';
  static LOADING_ERROR = 'An error occurred loading data.';
  static NAME_VALIDATION_MESSAGE = 'Only letters, digits, underscores(_), hyphens(-), and dots(.) are allowed and may not start with underscore or dot.';

  static get UNRESOLVED() {
    return new Promise(() => {
      // Intentionally left blank so the promise is never resolved for tests
    });
  }

  /**
   * Helper function to manage XState machine service lifecycle in tests.
   * Ensures service is properly started and stopped regardless of test outcome.
   *
   * @param {Object} machine - The XState machine to interpret
   * @param {Function} testFn - The test function that receives the service
   * @returns {Promise} The result of the test function
   *
   * @example
   * await TestUtils.withTestMachine(MyMachine, async (service) => {
   *   const finalState = await waitForState(service, (state) => state.matches('loaded'));
   *   expect(finalState.context.data).toBe(testData);
   * });
   */
  static async withTestMachine(machine, testFn) {
    const service = interpret(machine);
    try {
      service.start();
      return await testFn(service);
    } finally {
      service.stop();
    }
  };

  /**
   * @deprecated call render directly and use the {selectors} in this class instead
   */
  static render(view, extraSelectors) {
    const selectors = render(view);
    const {queryByText} = selectors;
    return {
      ...selectors,
      loadingMask: () => queryByText('Loading…'),
      savingMask: () => queryByText(/saving/i),
      submittingMask: () => queryByText(/submitting/i),
      ...extraSelectors(selectors)
    };
  }

  static async changeField(fieldSelector, value) {
    fireEvent.change(fieldSelector(), {
      currentTarget: {
        name: fieldSelector().name,
        value
      },
      target: {
        name: fieldSelector().name,
        value
      }
    });
    try {
      await waitFor(() => expect(fieldSelector()).toHaveValue(value));
    } catch(error) {
      throw new Error(`${fieldSelector().name} with value ${fieldSelector().value} did not match the expected value \n ${error.message}`);
    }
  }

  static selectors = {
    queryLoadingMask: () => screen.queryByText(/loading/i),
    querySavingMask: () => screen.queryByText(/saving/i),
    querySubmittingMask: () => screen.queryByText(/submitting/i),
    queryLoadError: (message) => {
      const options = {selector: '.nx-load-error__message'};
      if (message) {
        return screen.queryByText(`An error occurred loading data. ${message}`, options);
      }
      else {
        return screen.queryByText(/an error occurred loading data/i, options);
      }
    }
  }

  static formSelectors = {
    queryTitle: () => {
      const headings = screen.queryAllByRole('heading', {level: 1});
      // Return the Default UI heading (with text content), skip any empty Preview UI headings
      return headings.find(h => h.textContent?.trim()) || headings[0] || null;
    },
    querySubmitButton: () => screen.queryByRole('button', {name: SAVE_BUTTON_LABEL}),
    queryDiscardButton: () => screen.queryByRole('button', {name: DISCARD_BUTTON_LABEL}),
    queryFormError: (message) => {
      const options = {selector: '.nx-form--show-validation-errors.nx-form--has-validation-errors .nx-form__validation-errors .nx-alert__content'};
      if (message) {
        return screen.queryByText(`${TestUtils.THERE_WERE_ERRORS} ${message}`, options);
      }
      else {
        return screen.queryByText(/there were validation errors/i, options);
      }
    },
    querySaveError: (message) => {
      const options = {selector: '.nx-alert__content .nx-load-error__message'};
      if (message) {
        return screen.queryByText(`${TestUtils.SAVE_ERROR} ${message}`, options);
      } else {
        return screen.queryByText(new RegExp(TestUtils.SAVE_ERROR), options);
      }
    },
    querySaveErrorAlert: () => screen.queryByLabelText("form saving errors")
  };

  static tableSelectors = {
    table: () => screen.getByRole('table'),
    headerCells: () => screen.getAllByRole('columnheader'),
    headerCell: (name) => screen.getByRole('columnheader', {name}),
    tableBody: () => screen.getAllByRole('rowgroup')[1],
    rows: () => within(screen.getAllByRole('rowgroup')[1]).getAllByRole('row'),
    tableAlert: () => within(screen.getAllByRole('rowgroup')[1]).getByRole('alert'),
  };

  static expectTableHeaders(names) {
    const cells = TestUtils.tableSelectors.headerCells();
    names.forEach((name, index) => {
      expect(cells[index]).toHaveTextContent(name);
    });
  }

  static expectTableRows(data, keys) {
    const rows = TestUtils.tableSelectors.rows();

    expect(rows).toHaveLength(data.length);

    data.forEach((item, index) => {
      const row = rows[index];

      keys.forEach(key => expect(Object.hasOwn(item, key)).toBeTruthy());

      Object.values(pick(keys, item)).forEach((value, index)  => {
        expect(row.cells[index]).toHaveTextContent(value || '');
      })
    });
  }

  static expectProperRowsOrder(data, fieldName = 'name', columnIndex = 0) {
    const rows = TestUtils.tableSelectors.rows();
    data.forEach((item, index) => {
      expect(rows[index].cells[columnIndex]).toHaveTextContent(item[fieldName]);
    });
  }

  static async expectProperFilteredItemsCount(filter, query, count) {
    userEvent.clear(filter());
    userEvent.type(filter(), query);
    expect(TestUtils.tableSelectors.rows()).toHaveLength(count);
  }

  static async expectToSeeTooltipOnHover(element, tooltipMessage) {
    let tooltip = screen.queryByRole('tooltip', {name: tooltipMessage});
    expect(tooltip).not.toBeInTheDocument();

    userEvent.hover(element);
    tooltip = await screen.findByRole('tooltip', {name: tooltipMessage});
    expect(tooltip).toBeInTheDocument();

    userEvent.unhover(element);
    await waitForElementToBeRemoved(tooltip);
  }

  static makeExtResult(data, total = 0) {
    return {
      result: {
        data,
        success: true,
        total
      }
    };
  }

  static get XSS_STRING() {
    return 'XSS!<img src="/static/rapture/resources/icons/x16/user.png" onload="alert(0)">';
  }
}
