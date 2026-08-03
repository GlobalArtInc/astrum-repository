

import React from 'react';
import { screen, within, cleanup, render } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { memoryLocationPlugin, UIRouter, UIView } from '@uirouter/react';
import { createRouter } from '../../../router/createRouter';
import ExtJS from '../../../interface/ExtJS';
import UIStrings from '../../../constants/UIStrings';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';

import CapabilitiesCreate from './CapabilitiesCreate';

jest.mock('../../../interface/ExtJS');

// Mock ExtAPIUtils for DynamicFormField
jest.mock('../../../interface/ExtAPIUtils', () => ({
  extAPIRequest: jest.fn(),
  checkForErrorAndExtract: jest.fn(response => response.data),
  checkForError: jest.fn(),
  extractResult: jest.fn((response, defaultValue) => response.data || defaultValue),
}));

const { CAPABILITIES } = UIStrings;

const CAPABILITY_TYPES = [
  {
    id: 'audit',
    name: 'Audit',
    about: '<p>Audit capability for tracking repository access and changes.</p>',
    // No formFields - form with no fields
  },
  {
    id: 'baseurl',
    name: 'Base URL',
    about: '<p>Base URL configuration for the repository.</p>',
    formFields: [
      {
        id: 'baseUrl',
        type: 'url',
        label: 'Base URL',
        helpText: 'The base URL for the repository. Example: https://example.com',
        required: true,
        disabled: false,
        readOnly: false,
        initialValue: null,
        attributes: {},
      },
    ],
  },
  {
    id: 'scheduler',
    name: 'Scheduler',
    about: '<p>Scheduler capability for managing tasks and jobs.</p>',
    formFields: [
      {
        id: 'taskName',
        type: 'string',
        label: 'Task Name',
        helpText: 'Name of the scheduled task',
        required: true,
        disabled: false,
        readOnly: false,
        initialValue: null,
        attributes: {},
      },
      {
        id: 'interval',
        type: 'number',
        label: 'Interval (hours)',
        helpText: 'How often to run the task in hours',
        required: true,
        disabled: false,
        readOnly: false,
        initialValue: '12',
        attributes: {
          minValue: '1',
          maxValue: '24',
        },
      },
      {
        id: 'enabled',
        type: 'checkbox',
        label: 'Enable Task',
        helpText: 'Whether the task should be enabled',
        required: false,
        disabled: false,
        readOnly: false,
        initialValue: false,
        attributes: {},
      },
    ],
  },
  {
    id: 'zebra',
    name: 'Zebra',
    about: '<p>Zebra capability for testing sorting.</p>',
    formFields: [
      {
        id: 'description',
        type: 'text-area',
        label: 'Description',
        helpText: 'Description of the capability',
        required: false,
        disabled: false,
        readOnly: false,
        initialValue: null,
        attributes: {},
      },
    ],
  },
  {
    id: 'alpha',
    name: 'Alpha',
    about: '<p>Alpha capability for testing sorting.</p>',
    formFields: [
      {
        id: 'password',
        type: 'password',
        label: 'Password',
        helpText: 'Password for authentication',
        required: true,
        disabled: false,
        readOnly: false,
        initialValue: null,
        attributes: {},
      },
    ],
  },
];

// Dummy component for testing navigation
const DummyListPage = () => (
  <div>
    <h1>Capabilities List Page</h1>
  </div>
);

// Create a test router with capabilities routes
const createTestRouter = () => {
  const initialRoute = 'admin.system.capabilities.create';
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
      component: DummyListPage,
      data: { visibilityRequirements: {} },
    },
    {
      name: 'admin.system.capabilities.create',
      url: '/create',
      component: CapabilitiesCreate,
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

describe('CapabilitiesCreate', () => {
  let mockExtAPIRequest;

  const defaultMockImplementation = (action, method) => {
    if (method === 'readTypes') {
      return Promise.resolve({ data: CAPABILITY_TYPES });
    }
    // For CREATE or other methods, return success by default
    return Promise.resolve({ data: { success: true } });
  };

  beforeEach(() => {
    jest.clearAllMocks();

    // Restore any existing spies first to ensure clean state
    if (mockExtAPIRequest) {
      try {
        mockExtAPIRequest.mockRestore();
      } catch {
        // Ignore if already restored
      }
    }

    // Create fresh spies on the ExtAPIUtils methods
    mockExtAPIRequest = jest.spyOn(ExtAPIUtils, 'extAPIRequest');
    jest.spyOn(ExtAPIUtils, 'checkForErrorAndExtract').mockImplementation(response => response.data);
    jest.spyOn(ExtAPIUtils, 'checkForError').mockImplementation(() => {});
    jest
      .spyOn(ExtAPIUtils, 'extractResult')
      .mockImplementation((response, defaultValue) => response?.data || defaultValue);

    // Mock successful API response with capability types by default
    // This will handle both READ_TYPES and CREATE calls
    mockExtAPIRequest.mockImplementation(defaultMockImplementation);

    // Default permission check to true
    ExtJS.checkPermission.mockReturnValue(true);
  });

  afterEach(() => {
    cleanup();
  });

  const renderView = () => {
    const router = createTestRouter();
    router.plugin(memoryLocationPlugin);
    // Seed the starting URL that corresponds to the create state
    // This ensures each test starts with a clean router state at the create route
    router.urlService.url('admin/system/capabilities/create');

    const view = (
      <UIRouter router={router}>
        <UIView />
      </UIRouter>
    );
    return { ...render(view, () => ({})), router };
  };

  it('renders the page title and description', async () => {
    renderView();

    const heading = await screen.findByRole('heading', { name: CAPABILITIES.CREATE.TITLE });
    expect(heading).toBeVisible();
    expect(screen.getByText(CAPABILITIES.CREATE.DESCRIPTION)).toBeVisible();
  });

  it('renders the form title', async () => {
    renderView();

    const formTitle = await screen.findByText(CAPABILITIES.CREATE.FORM.TITLE);
    expect(formTitle).toBeVisible();
  });

  it('renders the select type label and dropdown', async () => {
    renderView();
    const select = await screen.findByRole('combobox');
    expect(select).toBeVisible();
    expect(select).toHaveAccessibleName(CAPABILITIES.CREATE.FORM.SELECT_TYPE);
  });

  it('renders all capability types in the dropdown', async () => {
    renderView();

    const select = await screen.findByRole('combobox');

    // Check placeholder option
    expect(within(select).getByRole('option', { name: CAPABILITIES.CREATE.FORM.SELECT_TYPE })).toBeInTheDocument();

    // Check all capability types are present
    CAPABILITY_TYPES.forEach(type => {
      expect(within(select).getByRole('option', { name: type.name })).toBeVisible();
    });
  });

  it('displays selected type details when a type is selected', async () => {
    renderView();

    const select = await screen.findByRole('combobox');

    // Select a capability type
    userEvent.selectOptions(select, 'audit');

    // Wait for the selected type details to appear
    const selectedType = CAPABILITY_TYPES.find(t => t.id === 'audit');
    const heading = screen.getByRole('heading', { name: selectedType.name });
    expect(heading).toBeVisible();

    // Verify the "about" content is rendered
    expect(screen.getByText(/Audit capability for tracking/i)).toBeVisible();
  });

  it('handles type selection change', async () => {
    renderView();

    const select = await screen.findByRole('combobox');

    // Select first type
    userEvent.selectOptions(select, 'audit');

    expect(screen.getByRole('heading', { name: CAPABILITY_TYPES[0].name })).toBeVisible();

    // Select different type
    userEvent.selectOptions(select, 'baseurl');

    // Previous type should no longer be visible
    expect(screen.queryByRole('heading', { name: CAPABILITY_TYPES[0].name })).not.toBeInTheDocument();
    // New type should be visible
    expect(screen.getByRole('heading', { name: CAPABILITY_TYPES[1].name })).toBeVisible();
  });

  it('shows loading state initially', async () => {
    // Delay the API response to see loading state
    mockExtAPIRequest.mockImplementation(
      () => new Promise(resolve => setTimeout(() => resolve({ data: CAPABILITY_TYPES }), 100))
    );

    renderView();

    // Should show loading state
    const loadingText = await screen.findByText(/loading/i);
    expect(loadingText).toBeVisible();
  });

  it('shows error state when API call fails', async () => {
    const errorMessage = 'Failed to load capability types';
    mockExtAPIRequest.mockRejectedValue(new Error(errorMessage));

    renderView();

    // Wait for error state to appear
    await screen.findByText(/An error occurred loading data/i);

    // Verify error message is displayed
    expect(screen.getByText(/Failed to load capability types/)).toBeVisible();
  });

  it('shows retry button when error occurs', async () => {
    const errorMessage = 'Failed to load capability types';
    mockExtAPIRequest.mockRejectedValue(new Error(errorMessage));

    renderView();

    // Wait for error state
    await screen.findByText(/An error occurred loading data/i);

    // Verify retry button is present
    const retryButton = screen.getByRole('button', { name: /retry/i });
    expect(retryButton).toBeVisible();
  });

  it('retries loading on retry button click', async () => {
    const errorMessage = 'Failed to load capability types';

    // First call fails
    mockExtAPIRequest.mockRejectedValueOnce(new Error(errorMessage));
    // Second call succeeds
    mockExtAPIRequest.mockResolvedValueOnce({ data: CAPABILITY_TYPES });

    renderView();

    // Wait for error state
    const errorText = await screen.findByText(/An error occurred loading data/i);
    expect(errorText).toBeVisible();

    // Click retry button
    const retryButton = screen.getByRole('button', { name: /retry/i });
    userEvent.click(retryButton);

    // Wait for successful load
    const select = await screen.findByRole('combobox');
    const newErrorText = screen.queryByText(/An error occurred loading data/i);
    expect(newErrorText).not.toBeInTheDocument();
    expect(select).toBeVisible();
  });

  it('handles cancel button click', async () => {
    renderView();

    const cancelButton = await screen.findByRole('button', { name: /cancel/i });

    userEvent.click(cancelButton);

    // Verify navigation to list page
    await screen.findByRole('heading', { name: /Capabilities List Page/i });
  });

  it('renders dangerouslySetInnerHTML correctly for type about content', async () => {
    renderView();

    const select = await screen.findByRole('combobox');

    // Select a type with HTML content
    userEvent.selectOptions(select, 'audit');

    const selectedType = CAPABILITY_TYPES.find(t => t.id === 'audit');
    expect(screen.getByRole('heading', { name: selectedType.name })).toBeVisible();

    // The HTML content should be rendered as raw HTML
    // We can verify by checking for text that appears in the HTML
    expect(screen.getByText(/Audit capability for tracking repository access and changes/i)).toBeVisible();
  });

  it('handles empty types list', async () => {
    mockExtAPIRequest.mockResolvedValue({ data: [] });

    renderView();

    const select = await screen.findByRole('combobox');

    // Should have placeholder option only
    const options = within(select).getAllByRole('option');
    expect(options).toHaveLength(1);
    expect(options[0]).toHaveTextContent(CAPABILITIES.CREATE.FORM.SELECT_TYPE);
  });

  it('enables placeholder option initially and disables after selection', async () => {
    renderView();

    const select = await screen.findByRole('combobox');

    // Initially, placeholder should be enabled
    let placeholderOption = within(select).getByRole('option', { name: CAPABILITIES.CREATE.FORM.SELECT_TYPE });
    expect(placeholderOption).not.toBeDisabled();

    // Select a type
    userEvent.selectOptions(select, 'audit');

    // After selection, placeholder should be disabled
    placeholderOption = within(select).getByRole('option', { name: CAPABILITIES.CREATE.FORM.SELECT_TYPE });
    expect(placeholderOption).toBeDisabled();
  });

  describe('type sorting', () => {
    it('sorts types alphabetically by name in the dropdown', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      const options = within(select).getAllByRole('option');

      // Skip the placeholder option (index 0)
      const typeOptions = options.slice(1);

      // Verify types are sorted alphabetically: Alpha, Audit, Base URL, Scheduler, Zebra
      expect(typeOptions[0]).toHaveTextContent('Alpha');
      expect(typeOptions[1]).toHaveTextContent('Audit');
      expect(typeOptions[2]).toHaveTextContent('Base URL');
      expect(typeOptions[3]).toHaveTextContent('Scheduler');
      expect(typeOptions[4]).toHaveTextContent('Zebra');
    });
  });

  describe('dynamic form fields', () => {
    it('renders form fields when a type with formFields is selected', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'baseurl');

      expect(screen.getByRole('heading', { name: 'Base URL' })).toBeVisible();

      // Verify form field is rendered
      const baseUrlField = screen.getByLabelText('Base URL');
      expect(baseUrlField).toBeVisible();
      const helpText = screen.getByText(/The base URL for the repository/i);
      expect(helpText).toBeVisible();
    });

    it('renders multiple form fields for types with multiple fields', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'scheduler');

      expect(screen.getByRole('heading', { name: 'Scheduler' })).toBeVisible();

      // Verify all form fields are rendered
      const taskNameField = screen.getByLabelText('Task Name');
      expect(taskNameField).toBeVisible();

      const intervalField = screen.getByLabelText('Interval (hours)');
      expect(intervalField).toBeVisible();
      expect(intervalField).toHaveValue('12'); // Verify initial value

      const checkboxField = screen.getByRole('checkbox', { name: /Enable Task/i });
      expect(checkboxField).toBeVisible();
    });

    it('does not render form fields section when type has no formFields', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'audit');

      expect(screen.getByRole('heading', { name: 'Audit' })).toBeVisible();

      // Verify no form fields are rendered - check that no field labels appear
      const formFields = screen.queryByTestId('form-fields');
      expect(formFields).not.toBeInTheDocument();
    });
  });

  describe('form validation and submission', () => {
    it('does not show validation errors before submit is attempted', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'baseurl');

      expect(screen.queryByRole('button', { name: /save/i })).toBeVisible();
      const errorAlert = screen.queryByText(/This field is required/i);
      expect(errorAlert).not.toBeInTheDocument();
    });

    it('shows validation errors after submit is attempted', async () => {
      renderView();
      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'baseurl');
      // Attempt to submit without filling required field
      const submitButton = screen.getByRole('button', { name: /save/i });
      expect(screen.queryByText(/This field is required/i)).not.toBeInTheDocument();

      userEvent.click(submitButton);
      expect(screen.getByText(/This field is required/i)).toBeVisible();
    });

    it('hides validation errors when form becomes valid after submit attempt', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'baseurl');

      // Attempt to submit without filling required field
      const submitButton = screen.getByRole('button', { name: /save/i });
      userEvent.click(submitButton);

      expect(screen.getByText(/This field is required/i)).toBeVisible();

      // Fill in the required field with valid URL
      const baseUrlField = screen.getByLabelText('Base URL');
      userEvent.type(baseUrlField, 'https://example.com');
      expect(screen.queryByText(/This field is required/i)).not.toBeInTheDocument();
      userEvent.click(submitButton);
      expect(await screen.findByRole('heading', { name: 'Capabilities List Page' })).toBeVisible();
    });

    it('resets submitAttempted when type changes', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'baseurl');

      const submitButton = screen.getByRole('button', { name: /save/i });
      userEvent.click(submitButton);
      expect(screen.getByText(/This field is required/i)).toBeVisible();

      // Change type
      userEvent.selectOptions(select, 'audit');

      expect(screen.queryByText(/This field is required/i)).not.toBeInTheDocument();
    });

    it('shows URL validation error for invalid URL format', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'baseurl');

      // Fill in invalid URL
      const baseUrlField = screen.getByLabelText('Base URL');
      userEvent.type(baseUrlField, 'invalid-url');

      // Attempt to submit
      const submitButton = screen.getByRole('button', { name: /save/i });
      userEvent.click(submitButton);

      expect(screen.queryByText(/URL should be in the format/i)).toBeVisible();
      userEvent.clear(baseUrlField);
      userEvent.type(baseUrlField, 'https://example.com');
      expect(screen.queryByText(/URL should be in the format/i)).not.toBeInTheDocument();

      userEvent.click(submitButton);
      expect(await screen.findByRole('heading', { name: 'Capabilities List Page' })).toBeVisible();
    });
  });

  describe('pristine state handling', () => {
    it('allows pristine save for types with no fields', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'audit');

      const submitButton = screen.getByRole('button', { name: /save/i });
      userEvent.click(submitButton);
      expect(await screen.findByRole('heading', { name: 'Capabilities List Page' })).toBeVisible();
    });

    it('allows pristine save for types with all optional fields', async () => {
      renderView();

      const select = await screen.findByRole('combobox');
      userEvent.selectOptions(select, 'zebra');

      const submitButton = screen.getByRole('button', { name: /save/i });
      userEvent.click(submitButton);
      expect(await screen.findByRole('heading', { name: 'Capabilities List Page' })).toBeVisible();
    });
  });
});
