

import React from 'react';
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import TestUtils from '../../../interface/TestUtils';
import { memoryLocationPlugin, UIRouter, UIView } from '@uirouter/react';
import { createRouter } from '../../../router/createRouter';
import ExtJS from '../../../interface/ExtJS';
import Permissions from '../../../constants/Permissions';

import CapabilitiesList from './CapabilitiesList';
import UIStrings from '../../../constants/UIStrings';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';
import { CAPABILITIES_TEST_DATA } from './Capabilities.testdata';

jest.mock('../../../interface/ExtJS');

const { CAPABILITIES } = UIStrings;
const { COLUMNS } = CAPABILITIES.LIST;

// Dummy component for testing navigation
const DummyEditPage = () => (
  <div>
    <h1>Edit Capability Page</h1>
  </div>
);

// Create a test router with capabilities routes
const createTestRouter = () => {
  const initialRoute = 'admin.system.capabilities.list';
  const menuRoutes = [
    {
      name: 'admin',
      url: 'admin',
      component: UIView,
      abstract: true,
      data: { visibilityRequirements: {} },
    },
    {
      name: 'admin.system',
      url: '/system',
      component: UIView,
      abstract: true,
      data: { visibilityRequirements: {} },
    },
    {
      name: 'admin.system.capabilities',
      url: '/capabilities',
      component: UIView,
      abstract: true,
      data: { visibilityRequirements: {} },
    },
    {
      name: 'admin.system.capabilities.list',
      url: '',
      component: CapabilitiesList,
      data: { visibilityRequirements: {} },
    },
    {
      name: 'admin.system.capabilities.edit',
      url: '/edit/:id',
      component: DummyEditPage,
      params: {
        id: { type: 'string' },
      },
      data: { visibilityRequirements: {} },
    },
  ];

  const missingRoute = {
    name: 'missing.route',
    url: 404,
    component: () => <div>Missing Route</div>,
    data: { visibilityRequirements: {} },
  };

  return createRouter({ initialRoute, menuRoutes, missingRoute });
};

describe('CapabilitiesList', () => {
  let mockExtAPIRequest;

  beforeEach(() => {
    jest.clearAllMocks();

    // Create a spy on the ExtAPIUtils methods
    mockExtAPIRequest = jest.spyOn(ExtAPIUtils, 'extAPIRequest');
    jest.spyOn(ExtAPIUtils, 'checkForErrorAndExtract').mockImplementation(response => response.data);

    // Mock successful API response with real data for more realistic testing
    mockExtAPIRequest.mockResolvedValue({ data: CAPABILITIES_TEST_DATA.REAL_DATA });

    // Default permission check to true (reset the mock to default behavior)
    ExtJS.checkPermission.mockReturnValue(true);
  });

  const renderView = () => {
    const router = createTestRouter();
    router.plugin(memoryLocationPlugin);
    // Seed the starting URL that corresponds to the list state
    router.urlService.url('admin/system/capabilities');

    const view = (
      <UIRouter router={router}>
        <UIView />
      </UIRouter>
    );
    return TestUtils.render(view, () => ({}));
  };

  it('renders the page title', async () => {
    renderView();
    const heading = await screen.findByRole('heading', { name: CAPABILITIES.MENU.text });
    expect(heading).toBeVisible();
    expect(screen.getByText(CAPABILITIES.MENU.description)).toBeVisible();
  });

  it('renders the table headers correctly', async () => {
    renderView();
    await waitForDataToLoad();
    assertAllHeaders(false);
  });

  it('renders the resolved data', async () => {
    renderView();

    // wait for data to have been loaded
    await waitForDataToLoad();
    assertAllHeaders(false);

    // check all rows (data is sorted by typeName ascending by default)
    const rows = screen.getAllByRole('row');
    const dataRows = rows.slice(1); // Skip header row
    expect(dataRows).toHaveLength(4); // Real data has 4 items

    const row1Cells = within(dataRows[0]).getAllByRole('cell');
    assertContainsExactText(row1Cells[0], 'Audit');
    assertContainsExactText(row1Cells[1], 'active');
    assertContainsExactText(row1Cells[2], 'Audit');
    assertContainsExactText(row1Cells[3], 'Enabled');
    assertContainsExactText(row1Cells[4], '-'); // Empty notes

    const row2Cells = within(dataRows[1]).getAllByRole('cell');
    assertContainsExactText(row2Cells[0], 'Base URL');
    assertContainsExactText(row2Cells[1], 'active');
    assertContainsExactText(row2Cells[2], 'Core');
    assertContainsExactText(row2Cells[3], '-'); // Null description
    assertContainsExactText(row2Cells[4], '-'); // Empty notes

    const row3Cells = within(dataRows[2]).getAllByRole('cell');
    assertContainsExactText(row3Cells[0], 'Scheduler');
    assertContainsExactText(row3Cells[1], 'active');
    assertContainsExactText(row3Cells[2], 'Scheduling');
    assertContainsExactText(row3Cells[3], 'Started');
    assertContainsExactText(row3Cells[4], 'Automatically added on Wed Oct 15 14:53:08 COT 2025');

    const row4Cells = within(dataRows[3]).getAllByRole('cell');
    assertContainsExactText(row4Cells[0], 'Upgrade');
    assertContainsExactText(row4Cells[1], 'active');
    assertContainsExactText(row4Cells[2], 'Upgrade');
    assertContainsExactText(row4Cells[3], 'Enabled');
    assertContainsExactText(row4Cells[4], '-'); // Empty notes
  });

  it('renders the create button', async () => {
    renderView();

    const createButton = await screen.findByText(CAPABILITIES.LIST.CREATE_BUTTON);

    expect(createButton).not.toBeDisabled();
    expect(createButton).toHaveAttribute('data-analytics-id', 'as-create-capability');
    expect(createButton).toBeInTheDocument();
  });

  it('disables the create button when not enough permissions', async () => {
    ExtJS.checkPermission.mockImplementation((permission) => {
      if (permission === Permissions.CAPABILITIES.CREATE) {
        return false;
      }
      return true;
    });

    renderView();

    const createButton = await screen.findByText(CAPABILITIES.LIST.CREATE_BUTTON);

    // Verify the mock was called with the correct permission
    expect(ExtJS.checkPermission).toHaveBeenCalledWith(Permissions.CAPABILITIES.CREATE);
    expect(createButton).toBeDisabled();
  });

  it('handles missing description and notes gracefully', async () => {
    mockExtAPIRequest.mockResolvedValue({
      data: CAPABILITIES_TEST_DATA.DATA_WITH_NULL_VALUES,
    });

    renderView();

    await screen.findByText('Test Capability');

    const rows = screen.getAllByRole('row');
    // Data is sorted by typeName ascending
    const firstDataRow = rows[1]; // Another Test (alphabetically first)
    const row1Cells = within(firstDataRow).getAllByRole('cell');

    assertContainsExactText(row1Cells[0], 'Another Test');
    assertContainsExactText(row1Cells[2], 'Test');
    assertContainsExactText(row1Cells[3], '-');
  });

  it('navigates to edit page when row is clicked', async () => {
    renderView();

    await waitForDataToLoad();

    // Find the clickable button in the first data row
    const firstRow = screen.getAllByRole('row')[1];

    // Use await with userEvent.click to ensure the click is processed
    await userEvent.click(firstRow);

    // Wait for navigation to complete and verify the edit page is rendered
    await screen.findByRole('heading', { name: 'Edit Capability Page' });
    expect(screen.getByRole('heading', { name: 'Edit Capability Page' })).toBeVisible();
  });

  it('shows error state when API call fails', async () => {
    // Mock API failure
    const errorMessage = 'Failed to load capabilities';
    mockExtAPIRequest.mockRejectedValue(new Error(errorMessage));

    renderView();

    // Wait for error state to appear - the error message includes a prefix
    await screen.findByText(/An error occurred loading data\. Failed to load capabilities/);

    // Verify error message is displayed
    expect(screen.getByText(/An error occurred loading data\. Failed to load capabilities/)).toBeInTheDocument();

    // Verify that no data rows are shown (only error row should be present)
    const dataRows = screen.queryAllByRole('row').slice(1); // Skip header row
    expect(dataRows).toHaveLength(1); // Only the error row should be present
    expect(dataRows[0]).toHaveTextContent('An error occurred loading data. Failed to load capabilities');
  });

  it('shows retry button when error occurs', async () => {
    // Mock API failure
    const errorMessage = 'Failed to load capabilities';
    mockExtAPIRequest.mockRejectedValue(new Error(errorMessage));

    renderView();

    // Wait for error state to appear
    await screen.findByText(/An error occurred loading data\. Failed to load capabilities/);

    // Verify retry button is present
    const retryButton = screen.getByRole('button', { name: /retry/i });
    expect(retryButton).toBeVisible();
  });

  it('handles real production data correctly', async () => {
    // Use mixed data to test both real and synthetic data
    mockExtAPIRequest.mockResolvedValue({
      data: CAPABILITIES_TEST_DATA.MIXED_DATA,
    });

    renderView();

    await waitForDataToLoad();
    assertAllHeaders(false);

    const rows = screen.getAllByRole('row');
    const dataRows = rows.slice(1); // Skip header row
    expect(dataRows).toHaveLength(2);

    // Check the first row with data
    const row1Cells = within(rows[1]).getAllByRole('cell');
    assertContainsExactText(row1Cells[0], 'Audit');
    assertContainsExactText(row1Cells[1], 'active');
    assertContainsExactText(row1Cells[2], 'Audit');
    assertContainsExactText(row1Cells[3], 'Enabled');
    assertContainsExactText(row1Cells[4], '-');

    // check the second row
    const row2Cells = within(rows[2]).getAllByRole('cell');
    assertContainsExactText(row2Cells[0], 'Test Capability');
    assertContainsExactText(row1Cells[1], 'active');
    assertContainsExactText(row1Cells[2], 'Audit');
    assertContainsExactText(row2Cells[3], 'A test capability for comprehensive testing');
    assertContainsExactText(row2Cells[4], 'Test capability for mixed data testing');
  });


  it('handles different capability states correctly', async () => {
    mockExtAPIRequest.mockResolvedValue({
      data: CAPABILITIES_TEST_DATA.DATA_WITH_DIFFERENT_STATES,
    });

    renderView();

    await waitForDataToLoad();
    assertAllHeaders(false);

    const rows = screen.getAllByRole('row');
    const dataRows = rows.slice(1); // Skip header row
    expect(dataRows).toHaveLength(4);

    // Test active capability
    const row1Cells = within(dataRows[0]).getAllByRole('cell');
    assertContainsExactText(row1Cells[0], 'Active Capability');
    assertContainsIconAndText(row1Cells[1], 'check-circle', 'active');
    assertContainsExactText(row1Cells[2], 'Test');
    assertContainsExactText(row1Cells[3], 'Active capability');

    // Test inactive capability
    const row2Cells = within(dataRows[1]).getAllByRole('cell');
    assertContainsExactText(row2Cells[0], 'Disabled Capability');
    assertContainsIconAndText(row2Cells[1], 'minus-circle', 'disabled');
    assertContainsExactText(row2Cells[2], 'Test');
    assertContainsExactText(row2Cells[3], 'Capability is disabled');

    // Test error capability
    const row3Cells = within(dataRows[2]).getAllByRole('cell');
    assertContainsExactText(row3Cells[0], 'Error Capability');
    assertContainsIconAndText(row3Cells[1], 'exclamation-circle', 'error');
    assertContainsExactText(row3Cells[2], 'Test');
    assertContainsExactText(row3Cells[3], 'Capability with error');

    // Test disabled capability
    const row4Cells = within(dataRows[3]).getAllByRole('cell');
    assertContainsExactText(row4Cells[0], 'Inactive Capability');
    assertContainsIconAndText(row4Cells[1], 'exclamation-circle', 'passive');
    assertContainsExactText(row4Cells[2], 'Test');
    assertContainsExactText(row4Cells[3], 'Inactive capability');
  });

  it('handles system vs user capabilities correctly', async () => {
    mockExtAPIRequest.mockResolvedValue({
      data: CAPABILITIES_TEST_DATA.DATA_WITH_SYSTEM_CAPABILITIES,
    });

    renderView();

    await waitForDataToLoad();

    assertAllHeaders(false);
    const rows = screen.getAllByRole('row');
    const dataRows = rows.slice(1); // Skip header row
    expect(dataRows).toHaveLength(2);

    // Test system capability
    const row1Cells = within(dataRows[0]).getAllByRole('cell');
    assertContainsExactText(row1Cells[0], 'System Scheduler');
    assertContainsExactText(row1Cells[2], 'System');
    assertContainsExactText(row1Cells[3], 'System scheduler capability');

    // Test user capability
    const row2Cells = within(dataRows[1]).getAllByRole('cell');
    assertContainsExactText(row2Cells[0], 'User Defined');
    assertContainsExactText(row2Cells[2], 'User');
    assertContainsExactText(row2Cells[3], 'User defined capability - description');
  });

  it('should render a mix of rows with and without category data', async () => {
    const capabilitiesWithAndWithoutCategories = CAPABILITIES_TEST_DATA.DATA_WITH_SYSTEM_CAPABILITIES
        .map(entry =>  ({...entry, tags: null }))

    // add back one tag with a category
    capabilitiesWithAndWithoutCategories[1] = {
      ...capabilitiesWithAndWithoutCategories[1],
      tags: { Category: 'some-category'}
    }

    mockExtAPIRequest.mockResolvedValue({
      data: capabilitiesWithAndWithoutCategories,
    });

    renderView();

    await waitForDataToLoad();

    assertAllHeaders(false);

    const rows = screen.getAllByRole('row');
    const dataRows = rows.slice(1);
    const row1Cells = within(dataRows[0]).getAllByRole('cell');
    assertContainsExactText(row1Cells[0], 'System Scheduler');
    assertContainsExactText(row1Cells[1], 'active');
    assertContainsExactText(row1Cells[2], '-');
    assertContainsExactText(row1Cells[3], 'System scheduler capability');
    assertContainsExactText(row1Cells[4], 'System capability');

    const row2Cells = within(dataRows[1]).getAllByRole('cell');
    assertContainsExactText(row2Cells[0], 'User Defined');
    assertContainsExactText(row2Cells[1], 'active');
    assertContainsExactText(row2Cells[2], 'some-category');
    assertContainsExactText(row2Cells[3], 'User defined capability - description');
    assertContainsExactText(row2Cells[4], 'User defined capability - note');
  })

  it('should render row with repository set on tags', async () => {
    const capabilitiesWithSomeRepos = [...CAPABILITIES_TEST_DATA.DATA_WITH_SYSTEM_CAPABILITIES]

    // add in a repository
    capabilitiesWithSomeRepos[1] = {
      ...capabilitiesWithSomeRepos[1],
      tags: { ...capabilitiesWithSomeRepos[1].tags, Repository: 'some-repository'}
    }

    mockExtAPIRequest.mockResolvedValue({
      data: capabilitiesWithSomeRepos,
    });

    renderView();

    await waitForDataToLoad();

    assertAllHeaders(true);

    const rows = screen.getAllByRole('row');
    const dataRows = rows.slice(1);

    const row1Cells = within(dataRows[0]).getAllByRole('cell');

    assertContainsExactText(row1Cells[0], 'System Scheduler');
    assertContainsExactText(row1Cells[1], 'active');
    assertContainsExactText(row1Cells[2], 'System');
    assertContainsExactText(row1Cells[3], '-'); // repo not set on this row
    assertContainsExactText(row1Cells[4], 'System scheduler capability');
    assertContainsExactText(row1Cells[5], 'System capability');

    const row2Cells = within(dataRows[1]).getAllByRole('cell');
    assertContainsExactText(row2Cells[0], 'User Defined');
    assertContainsExactText(row2Cells[1], 'active');
    assertContainsExactText(row2Cells[2], 'User');
    assertContainsExactText(row2Cells[3], 'some-repository');
    assertContainsExactText(row2Cells[4], 'User defined capability - description');
    assertContainsExactText(row2Cells[5], 'User defined capability - note');
  })


  describe('filtering', () => {
    it('should filter by typeName', async () => {
      renderView();
      await waitForDataToLoad();

      const filterInput = getFilterInput();
      await userEvent.type(filterInput, 'Audit');

      // Should only show Audit capability
      const rows = screen.getAllByRole('row');
      const dataRows = rows.slice(1);
      expect(dataRows).toHaveLength(1);

      const row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[0], 'Audit');
    });

    it('should filter by description', async () => {
      renderView();
      await waitForDataToLoad();

      const filterInput = getFilterInput();
      await userEvent.type(filterInput, 'Started');

      // Should only show Scheduler capability with "Started" description
      const rows = screen.getAllByRole('row');
      const dataRows = rows.slice(1);
      expect(dataRows).toHaveLength(1);

      const row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[0], 'Scheduler');
      assertContainsExactText(row1Cells[3], 'Started');
    });

    it('should filter by notes', async () => {
      renderView();
      await waitForDataToLoad();

      const filterInput = getFilterInput();
      await userEvent.type(filterInput, 'Automatically');

      // Should only show Scheduler capability with notes containing "Automatically"
      const rows = screen.getAllByRole('row');
      const dataRows = rows.slice(1);
      expect(dataRows).toHaveLength(1);

      const row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[0], 'Scheduler');
      expect(row1Cells[4]).toHaveTextContent('Automatically');
    });

    it('should filter by tags.Category', async () => {
      renderView();
      await waitForDataToLoad();

      const filterInput = getFilterInput();
      await userEvent.type(filterInput, 'Scheduling');

      // Should only show capabilities with Scheduling category
      const rows = screen.getAllByRole('row');
      const dataRows = rows.slice(1);
      expect(dataRows).toHaveLength(1);

      const row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[2], 'Scheduling');
    });

    it('should filter by tags.Repository', async () => {
      const testDataWithRepo = [
        {
          ...CAPABILITIES_TEST_DATA.REAL_DATA[0],
          tags: {
            Category: 'Audit',
            Repository: 'maven-central',
          },
        },
        ...CAPABILITIES_TEST_DATA.REAL_DATA.slice(1),
      ];

      mockExtAPIRequest.mockResolvedValue({ data: testDataWithRepo });

      renderView();
      await waitForDataToLoad();

      const filterInput = getFilterInput();
      await userEvent.type(filterInput, 'maven-central');

      // Should only show capability with maven-central repository
      const rows = screen.getAllByRole('row');
      const dataRows = rows.slice(1);
      expect(dataRows).toHaveLength(1);

      const row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[0], 'Audit');
      assertContainsExactText(row1Cells[3], 'maven-central');
    });

    it('should be case insensitive', async () => {
      renderView();
      await waitForDataToLoad();

      const filterInput = getFilterInput();
      await userEvent.type(filterInput, 'audit');

      // Should show Audit capability despite lowercase search
      const rows = screen.getAllByRole('row');
      const dataRows = rows.slice(1);
      expect(dataRows).toHaveLength(1);

      const row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[0], 'Audit');
    });

    it('should show no rows when no matches found', async () => {
      renderView();
      await waitForDataToLoad();

      const filterInput = getFilterInput();
      await userEvent.type(filterInput, 'NonExistentCapability');

      // Should show no data rows (only header)
      const rows = screen.getAllByRole('row');
      expect(rows).toHaveLength(1); // Only header row
    });

    it('should handle filtering with partial matches', async () => {
      renderView();
      await waitForDataToLoad();

      const filterInput = getFilterInput();
      await userEvent.type(filterInput, 'Sch');

      // Should show Scheduler capability
      const rows = screen.getAllByRole('row');
      const dataRows = rows.slice(1);
      expect(dataRows).toHaveLength(1);

      const row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[0], 'Scheduler');
    });
  });

  async function waitForDataToLoad() {
    await screen.findAllByRole('row')
  }

  function assertAllHeaders(hasRepo) {
    const select = "Select Row";
    const rows = screen.getAllByRole('row');
    const headerCells = within(rows[0]).getAllByRole('columnheader');

    if (hasRepo) {
      expect(headerCells).toHaveLength(7);
      assertContainsExactText(headerCells[0], COLUMNS.TYPE)
      assertContainsExactText(headerCells[1], COLUMNS.STATE)
      assertContainsExactText(headerCells[2], COLUMNS.CATEGORY)
      assertContainsExactText(headerCells[3], COLUMNS.REPOSITORY)
      assertContainsExactText(headerCells[4], COLUMNS.DESCRIPTION)
      assertContainsExactText(headerCells[5], COLUMNS.NOTES)
      assertContainsExactText(headerCells[6], select)
    } else {
      expect(headerCells).toHaveLength(6);
      assertContainsExactText(headerCells[0], COLUMNS.TYPE)
      assertContainsExactText(headerCells[1], COLUMNS.STATE)
      assertContainsExactText(headerCells[2], COLUMNS.CATEGORY)
      assertContainsExactText(headerCells[3], COLUMNS.DESCRIPTION)
      assertContainsExactText(headerCells[4], COLUMNS.NOTES)
      assertContainsExactText(headerCells[5], select)
    }
  }

  function assertContainsExactText(htmlElement, expectedText) {
    expect(htmlElement).toHaveTextContent(new RegExp(`^${expectedText}$`));
  }

  function assertContainsIconAndText(htmlElement, expectedIcon, expectedText) {
    assertContainsExactText(htmlElement, expectedText);
    const icon = within(htmlElement).getByRole("img", { hidden: true });
    expect(icon).toHaveAttribute("data-icon", expectedIcon);
  }

  function getFilterInput() {
    const filterInput = document.querySelector('.as-filter-input input');
    expect(filterInput).toBeVisible();

    return filterInput;
  }

  describe('Sorting', () => {
    it('should sort by Type (typeName) ascending by default and allow toggling to descending', async () => {
      renderView();
      await waitForDataToLoad();

      // Verify default sort is typeName ascending
      let rows = screen.getAllByRole('row');
      let dataRows = rows.slice(1);

      // Default order should be typeName ascending: Audit, Base URL, Scheduler, Upgrade
      let row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[0], 'Audit');

      let row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[0], 'Base URL');

      let row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[0], 'Scheduler');

      let row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[0], 'Upgrade');

      // Click Type header to toggle to descending
      const typeHeader = screen.getByText(COLUMNS.TYPE).closest('button');
      await userEvent.click(typeHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify descending order: Upgrade, Scheduler, Base URL, Audit
      rows = screen.getAllByRole('row');
      dataRows = rows.slice(1);

      row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[0], 'Upgrade');

      row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[0], 'Scheduler');

      row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[0], 'Base URL');

      row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[0], 'Audit');
    });

    it('should sort by State ascending and descending', async () => {
      // Use data with different states to test sorting
      mockExtAPIRequest.mockResolvedValue({
        data: CAPABILITIES_TEST_DATA.DATA_WITH_DIFFERENT_STATES,
      });

      renderView();
      await waitForDataToLoad();

      // Click State header to sort ascending
      const stateHeader = screen.getByText(COLUMNS.STATE).closest('button');
      await userEvent.click(stateHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify ascending order: active, disabled, error, passive
      let rows = screen.getAllByRole('row');
      let dataRows = rows.slice(1);

      let row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsIconAndText(row1Cells[1], 'check-circle', 'active');

      let row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsIconAndText(row2Cells[1], 'minus-circle', 'disabled');

      let row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsIconAndText(row3Cells[1], 'exclamation-circle', 'error');

      let row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsIconAndText(row4Cells[1], 'exclamation-circle', 'passive');

      // Click State header again to sort descending
      await userEvent.click(stateHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify descending order: passive, error, disabled, active
      rows = screen.getAllByRole('row');
      dataRows = rows.slice(1);

      row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsIconAndText(row1Cells[1], 'exclamation-circle', 'passive');

      row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsIconAndText(row2Cells[1], 'exclamation-circle', 'error');

      row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsIconAndText(row3Cells[1], 'minus-circle', 'disabled');

      row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsIconAndText(row4Cells[1], 'check-circle', 'active');
    });

    it('should sort by Category ascending and descending', async () => {
      renderView();
      await waitForDataToLoad();

      // Click Category header to sort ascending
      const categoryHeader = screen.getByText(COLUMNS.CATEGORY).closest('button');
      await userEvent.click(categoryHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify ascending order: Audit, Core, Scheduling, Upgrade
      let rows = screen.getAllByRole('row');
      let dataRows = rows.slice(1);

      let row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[2], 'Audit');

      let row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[2], 'Core');

      let row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[2], 'Scheduling');

      let row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[2], 'Upgrade');

      // Click Category header again to sort descending
      await userEvent.click(categoryHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify descending order: Upgrade, Scheduling, Core, Audit
      rows = screen.getAllByRole('row');
      dataRows = rows.slice(1);

      row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[2], 'Upgrade');

      row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[2], 'Scheduling');

      row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[2], 'Core');

      row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[2], 'Audit');
    });

    it('should sort by Repository ascending and descending when column is visible', async () => {
      const capabilitiesWithRepos = [
        {
          ...CAPABILITIES_TEST_DATA.REAL_DATA[0],
          tags: { ...CAPABILITIES_TEST_DATA.REAL_DATA[0].tags, Repository: 'maven-central' },
        },
        {
          ...CAPABILITIES_TEST_DATA.REAL_DATA[1],
          tags: { ...CAPABILITIES_TEST_DATA.REAL_DATA[1].tags, Repository: 'npm-proxy' },
        },
        {
          ...CAPABILITIES_TEST_DATA.REAL_DATA[2],
          tags: { ...CAPABILITIES_TEST_DATA.REAL_DATA[2].tags, Repository: 'docker-hosted' },
        },
        {
          ...CAPABILITIES_TEST_DATA.REAL_DATA[3],
          tags: { ...CAPABILITIES_TEST_DATA.REAL_DATA[3].tags, Repository: 'releases' },
        },
      ];

      mockExtAPIRequest.mockResolvedValue({ data: capabilitiesWithRepos });

      renderView();
      await waitForDataToLoad();

      // Verify Repository column is visible
      assertAllHeaders(true);

      // Click Repository header to sort ascending
      const repositoryHeader = screen.getByText(COLUMNS.REPOSITORY).closest('button');
      await userEvent.click(repositoryHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify ascending order: docker-hosted, maven-central, npm-proxy, releases
      let rows = screen.getAllByRole('row');
      let dataRows = rows.slice(1);

      let row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[3], 'docker-hosted');

      let row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[3], 'maven-central');

      let row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[3], 'npm-proxy');

      let row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[3], 'releases');

      // Click Repository header again to sort descending
      await userEvent.click(repositoryHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify descending order: releases, npm-proxy, maven-central, docker-hosted
      rows = screen.getAllByRole('row');
      dataRows = rows.slice(1);

      row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[3], 'releases');

      row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[3], 'npm-proxy');

      row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[3], 'maven-central');

      row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[3], 'docker-hosted');
    });

    it('should sort by Description ascending and descending', async () => {
      renderView();
      await waitForDataToLoad();

      // Click Description header to sort ascending
      const descriptionHeader = screen.getByText(COLUMNS.DESCRIPTION).closest('button');
      await userEvent.click(descriptionHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify ascending order (null/-/empty first, then alphabetically)
      let rows = screen.getAllByRole('row');
      let dataRows = rows.slice(1);

      let row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[3], '-'); // Base URL has null description

      let row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[3], 'Enabled'); // Audit

      let row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[3], 'Enabled'); // Upgrade

      let row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[3], 'Started'); // Scheduler

      // Click Description header again to sort descending
      await userEvent.click(descriptionHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify descending order
      rows = screen.getAllByRole('row');
      dataRows = rows.slice(1);

      row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[3], 'Started');

      row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[3], 'Enabled'); // Could be Upgrade

      row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[3], 'Enabled'); // Could be Audit

      row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[3], '-');
    });

    it('should sort by Notes ascending and descending', async () => {
      renderView();
      await waitForDataToLoad();

      // Click Notes header to sort ascending
      const notesHeader = screen.getByText(COLUMNS.NOTES).closest('button');
      await userEvent.click(notesHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify ascending order (empty/- values first, then the one with text)
      let rows = screen.getAllByRole('row');
      let dataRows = rows.slice(1);

      // First three should have empty notes (-)
      let row1Cells = within(dataRows[0]).getAllByRole('cell');
      assertContainsExactText(row1Cells[4], '-');

      let row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[4], '-');

      let row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[4], '-');

      // Last one should be Scheduler with notes
      let row4Cells = within(dataRows[3]).getAllByRole('cell');
      expect(row4Cells[4]).toHaveTextContent('Automatically');

      // Click Notes header again to sort descending
      await userEvent.click(notesHeader);

      // Wait for sort to complete
      await screen.findAllByRole('row');

      // Verify descending order (text value first, then empty ones)
      rows = screen.getAllByRole('row');
      dataRows = rows.slice(1);

      row1Cells = within(dataRows[0]).getAllByRole('cell');
      expect(row1Cells[4]).toHaveTextContent('Automatically');

      row2Cells = within(dataRows[1]).getAllByRole('cell');
      assertContainsExactText(row2Cells[4], '-');

      row3Cells = within(dataRows[2]).getAllByRole('cell');
      assertContainsExactText(row3Cells[4], '-');

      row4Cells = within(dataRows[3]).getAllByRole('cell');
      assertContainsExactText(row4Cells[4], '-');
    });
  });
});
