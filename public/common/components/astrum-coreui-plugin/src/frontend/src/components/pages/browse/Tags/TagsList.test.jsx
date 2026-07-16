
import React from 'react';
import {render, screen, waitForElementToBeRemoved, within, waitFor} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import axios from 'axios';
import {sort, prop, descend, ascend} from 'ramda';
import TestUtils from '@globalart/astrum-ui-plugin/src/frontend/src/interface/TestUtils';

import TagsList from './TagsList';
import UIStrings from '../../../../constants/UIStrings';

describe('TagsList', function() {
  const tags = [
    {
      id: 'tag1',
      firstCreatedTime: '1/1/2020, 1:00:00 AM',
      lastUpdatedTime: '1/2/2020, 2:00:00 AM'
    },
    {
      id: 'tag2',
      firstCreatedTime: '2/2/2020, 1:00:00 AM',
      lastUpdatedTime: '2/3/2020, 2:00:00 AM'
    }
  ],
  emptyTag = [];

  afterEach(() => {
    jest.restoreAllMocks();
  });

  const FIELDS = {
    ID: 'id',
    FIRST_CREATED: 'firstCreatedTime',
    LAST_UPDATED: 'lastUpdatedTime'
  };

  const {NAME, FIRST_CREATED, LAST_UPDATED} = UIStrings.TAGS.LIST.COLUMNS;
  const {COLUMNS} = UIStrings.TAGS.LIST;

  const sortTags = (field, order = ascend) => sort(order(prop(field)), tags);

  const selectors = {
    ...TestUtils.selectors,
    ...TestUtils.tableSelectors,
    getFilterInput: () => screen.getByPlaceholderText('Filter by Name'),
    getEmptyMessage: () => screen.getByText('No tags found.')
  };

  async function renderView(data) {
    axios.post.mockResolvedValue({data: TestUtils.makeExtResult(data)});

    render(<TagsList/>);
    await waitForElementToBeRemoved(selectors.queryLoadingMask());
  };

  it('renders the resolved empty text', async function() {
    await renderView(emptyTag);
    await waitFor(() => {
      expect(selectors.getEmptyMessage()).toBeInTheDocument();
    });
  });

  it('renders the error message', async function() {
    axios.post.mockRejectedValue({message: 'Error'});
    render(<TagsList/>);
    await waitForElementToBeRemoved(selectors.queryLoadingMask());
    const error = selectors.tableAlert();

    expect(error).toBeInTheDocument();
    expect(error).toHaveTextContent('Error');
  });

  it('renders the resolved data', async function() {
    await renderView(tags);

    TestUtils.expectTableHeaders(Object.values(COLUMNS));
    TestUtils.expectTableRows(tags, Object.values(FIELDS));
  });

  it('renders a "Filter" text input', async function() {
    await renderView(tags);

    expect(selectors.getFilterInput()).toBeInTheDocument();
  });

  it('filters by the text value when the user types into the filter', async function() {
    await renderView(tags);

    await TestUtils.expectProperFilteredItemsCount(selectors.getFilterInput, '', tags.length);
    await TestUtils.expectProperFilteredItemsCount(selectors.getFilterInput, 'tag1', 1);
    await TestUtils.expectProperFilteredItemsCount(selectors.getFilterInput, 'tag2', 1);
  });

  it('unfilters when the clear button is pressed', async function() {
    await renderView(tags);

    await TestUtils.expectProperFilteredItemsCount(selectors.getFilterInput, '', tags.length);
    await TestUtils.expectProperFilteredItemsCount(selectors.getFilterInput, 'tag1', 1);

    const clearBtn = await screen.findByRole('button', {name: 'Clear filter'});

    userEvent.click(clearBtn);
    expect(selectors.rows()).toHaveLength(2);
  });

  it('unfilters when the ESC key is pressed', async function() {
    await renderView(tags);

    await TestUtils.expectProperFilteredItemsCount(selectors.getFilterInput, '', tags.length);
    await TestUtils.expectProperFilteredItemsCount(selectors.getFilterInput, 'tag1', 1);

    userEvent.type(selectors.getFilterInput(), '{esc}');
    expect(selectors.rows()).toHaveLength(2);
  });

  it('sorts the rows by name', async function() {
    await renderView(tags);

    const nameHeader = selectors.headerCell(NAME);

    TestUtils.expectProperRowsOrder(tags, FIELDS.ID);

    userEvent.click(nameHeader);
    let sortedTags = sortTags(FIELDS.ID, descend);
    TestUtils.expectProperRowsOrder(sortedTags, FIELDS.ID);

    userEvent.click(nameHeader);
    sortedTags = sortTags(FIELDS.ID);
    TestUtils.expectProperRowsOrder(sortedTags, FIELDS.ID);
  });

  it('sorts the rows by firstCreated', async function() {
    await renderView(tags);

    const firstCreatedHeader = selectors.headerCell(FIRST_CREATED);

    TestUtils.expectProperRowsOrder(tags, FIELDS.ID);

    userEvent.click(firstCreatedHeader);
    let sortedTags = sortTags(FIELDS.FIRST_CREATED);
    TestUtils.expectProperRowsOrder(sortedTags, FIELDS.ID);

    userEvent.click(firstCreatedHeader);
    sortedTags = sortTags(FIELDS.FIRST_CREATED, descend);
    TestUtils.expectProperRowsOrder(sortedTags, FIELDS.ID);
  });

  it('sorts the rows by lastUpdated', async function() {
    await renderView(tags);

    const lastUpdatedHeader = selectors.headerCell(LAST_UPDATED);

    TestUtils.expectProperRowsOrder(tags, FIELDS.ID);

    userEvent.click(lastUpdatedHeader);
    let sortedTags = sortTags(FIELDS.LAST_UPDATED);
    TestUtils.expectProperRowsOrder(sortedTags, FIELDS.ID);

    userEvent.click(lastUpdatedHeader);
    sortedTags = sortTags(FIELDS.LAST_UPDATED, descend);
    TestUtils.expectProperRowsOrder(sortedTags, FIELDS.ID);
  });

  it('renders tooltips of sorting direction when hovering', async function() {
    await renderView(tags);

    const headerBtn = within(selectors.headerCell(FIRST_CREATED)).getByRole('button');

    await TestUtils.expectToSeeTooltipOnHover(headerBtn, 'First created time unsorted');

    userEvent.click(headerBtn);
    await TestUtils.expectToSeeTooltipOnHover(headerBtn, 'First created time ascending');

    userEvent.click(headerBtn);
    await TestUtils.expectToSeeTooltipOnHover(headerBtn, 'First created time descending');
  });

  describe('pagination', function () {
    const manyTags = Array.from({ length: 60 }, (_, i) => ({
      id: `tag${String(i + 1).padStart(2, '0')}`,
      firstCreatedTime: `${i + 1}/1/2020, 1:00:00 AM`,
      lastUpdatedTime: `${i + 1}/2/2020, 2:00:00 AM`,
    }));

    it('renders pagination when there are more than 25 tags and correct number of rows', async function () {
      await renderView(manyTags);

      const pagination = screen.getByRole('navigation', { name: /pagination/i });
      const buttons = pagination.querySelectorAll('button');
      const rows = selectors.rows();
      expect(rows).toHaveLength(25);
      // first page, second page, last page, next page
      expect(buttons).toHaveLength(4);
      expect(pagination).toBeVisible();
    });

    it('does not render pagination when there are 25 or fewer tags', async function () {
      const fewTags = manyTags.slice(0, 25);
      await renderView(fewTags);

      const pagination = screen.queryByRole('navigation', { name: /pagination/i });
      expect(pagination).not.toBeInTheDocument();
    });

    it('displays first page by default', async function () {
      await renderView(manyTags);

      // Should show first 25 tags
      expect(screen.getByText('tag01')).toBeVisible();
      expect(screen.getByText('tag25')).toBeVisible();
      expect(screen.queryByText('tag26')).not.toBeInTheDocument();
    });

    it('navigates to second page when next button is clicked', async function () {
      await renderView(manyTags);

      const pagination = screen.getByRole('navigation', { name: /pagination/i });
      const nextPageButton = within(pagination).getByRole('button', { name: 'goto page 2' });

      // Should show tags 1-25
      expect(screen.getByText('tag01')).toBeVisible();
      expect(screen.getByText('tag25')).toBeVisible();
      expect(screen.queryByText('tag26')).not.toBeInTheDocument();
      expect(screen.queryByText('tag51')).not.toBeInTheDocument();

      userEvent.click(nextPageButton);

      // Should show tags 26-50
      expect(screen.getByText('tag26')).toBeVisible();
      expect(screen.getByText('tag50')).toBeVisible();
      expect(screen.queryByText('tag25')).not.toBeInTheDocument();
      expect(screen.queryByText('tag51')).not.toBeInTheDocument();
    });

    it('resets to first page when filter is applied', async function () {
      await renderView(manyTags);

      // Navigate to second page first (if possible)
      const pagination = screen.getByRole('navigation', { name: /pagination/i });
      const firstPageButton = within(pagination).getByRole('button', { name: 'goto first page' });
      const secondPageButton = within(pagination).getByRole('button', { name: 'goto page 2' });

      expect(firstPageButton).toHaveClass('selected');
      expect(secondPageButton).not.toHaveClass('selected');

      userEvent.click(secondPageButton);

      expect(firstPageButton).not.toHaveClass('selected');
      expect(secondPageButton).toHaveClass('selected');

      // Apply filter
      const filterInput = selectors.getFilterInput();
      userEvent.type(filterInput, 'tag');

      expect(firstPageButton).toHaveClass('selected');
      expect(secondPageButton).not.toHaveClass('selected');
    });

    it('maintains pagination state when sorting', async function () {
      await renderView(manyTags);

      const pagination = screen.getByRole('navigation', { name: /pagination/i });
      const firstPageButton = within(pagination).getByRole('button', { name: 'goto first page' });
      const lastPageButton = within(pagination).getByRole('button', { name: 'goto last page' });

      expect(firstPageButton).toHaveClass('selected');
      expect(lastPageButton).not.toHaveClass('selected');

      userEvent.click(lastPageButton);

      expect(firstPageButton).not.toHaveClass('selected');
      expect(lastPageButton).toHaveClass('selected');
      expect(screen.getByText('tag60')).toBeVisible();
      expect(screen.queryByText('tag01')).not.toBeInTheDocument();

      // Sort by name (descending)
      const nameHeader = selectors.headerCell(NAME);
      userEvent.click(nameHeader);

      // Should still be on last page
      expect(firstPageButton).not.toHaveClass('selected');
      expect(lastPageButton).toHaveClass('selected');
      expect(screen.queryByText('tag60')).not.toBeInTheDocument();
      expect(screen.getByText('tag01')).toBeVisible();
    });
  });
});
