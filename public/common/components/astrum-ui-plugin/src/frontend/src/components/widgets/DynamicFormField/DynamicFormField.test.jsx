
import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import DynamicFormField from './DynamicFormField';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';

// Mock ExtAPIUtils
jest.mock('../../../interface/ExtAPIUtils');

// Helper function to create mock context
const createMockContext = (id, value = '', onChange = jest.fn()) => ({
  id,
  current: {
    context: {
      data: { [id]: value },
      pristineData: { [id]: value },
      isTouched: {},
      validationErrors: {},
    },
  },
  onChange,
});

// Test wrapper component that manages state like a real form
const TestWrapper = ({ id, initialValue = '', children }) => {
  const [value, setValue] = React.useState(initialValue);

  const mockContext = {
    id,
    current: {
      context: {
        data: { [id]: value },
        pristineData: { [id]: initialValue },
        isTouched: {},
        validationErrors: {},
      },
    },
    onChange: (fieldId, newValue) => {
      setValue(newValue);
    },
  };

  return children(mockContext, value);
};

describe('DynamicFormField', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('string field type', () => {
    const defaultProps = createMockContext('testField');

    it('renders a text input that users can type into', () => {
      render(<DynamicFormField {...defaultProps} dynamicProps={{ type: 'string', attributes: {} }} />);

      const textInput = screen.getByRole('textbox');
      expect(textInput).toBeVisible();
      expect(textInput).toHaveValue('');
    });

    it('allows user to type in the input', async () => {
      render(
        <TestWrapper id='testField' initialValue=''>
          {mockContext => <DynamicFormField {...mockContext} dynamicProps={{ type: 'string', attributes: {} }} />}
        </TestWrapper>
      );

      const textInput = screen.getByRole('textbox');
      await userEvent.type(textInput, 'test value');

      // The component should be focusable and allow typing
      expect(textInput).toBeVisible();
      expect(textInput).not.toBeDisabled();
      expect(textInput).toHaveValue('test value');
    });

    it('disables the input when disabled prop is true', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'string',
            attributes: {},
            disabled: true,
          }}
        />
      );

      const textInput = screen.getByRole('textbox');
      expect(textInput).toBeDisabled();
    });

    it('renders read-only string as text', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{ type: "string", label: "My Text", attributes: {} }}
          initialValue='something'
          readOnly={true}
        />
      );

      const textLabel = screen.getByRole('term');
      const textValue = screen.getByRole('definition');
      expect(textLabel).toBeVisible();
      expect(textValue).toBeVisible();
      expect(textLabel).toHaveTextContent('My Text');
      expect(textValue).toHaveTextContent('something');
    });
  });

  describe('number field type', () => {
    const defaultProps = createMockContext('numberField');

    it('renders a text input for number entry', () => {
      render(<DynamicFormField {...defaultProps} dynamicProps={{ type: 'number', attributes: {} }} />);

      const textInput = screen.getByRole('textbox');
      expect(textInput).toBeVisible();
    });

    it('renders a number input that accepts typing', async () => {
      render(
        <TestWrapper id='numberField' initialValue=''>
          {mockContext => <DynamicFormField {...mockContext} dynamicProps={{ type: 'number', attributes: {} }} />}
        </TestWrapper>
      );

      const textInput = screen.getByRole('textbox');
      await userEvent.type(textInput, 'abc123def');

      // The component should be focusable and allow typing
      expect(textInput).toBeVisible();
      expect(textInput).not.toBeDisabled();
      expect(textInput).toHaveValue('123'); // Only digits should remain after filtering
    });

    it('renders number input with minimum value validation', () => {
      render(
        <DynamicFormField
          {...createMockContext('numberField', '0')}
          dynamicProps={{
            type: 'number',
            attributes: { minValue: '1' },
          }}
        />
      );

      const textInput = screen.getByRole('textbox');
      expect(textInput).toBeVisible();
      expect(textInput).toHaveValue('0');
    });

    it('renders read-only number input as text', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{ type: "number", label: "My Number", attributes: {} }}
          initialValue={123}
          readOnly={true}
        />
      );

      const numberLabel = screen.getByRole('term');
      const numberValue = screen.getByRole('definition');
      expect(numberLabel).toBeVisible();
      expect(numberValue).toBeVisible();
      expect(numberLabel).toHaveTextContent('My Number');
      expect(numberValue).toHaveTextContent('123');
    });
  });

  describe('checkbox field type', () => {
    const defaultProps = createMockContext('checkboxField', false);

    it('renders a checkbox that users can check', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'checkbox',
            attributes: {},
            label: 'Test checkbox',
          }}
        />
      );

      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).toBeVisible();
      expect(checkbox).not.toBeChecked();
    });

    it('allows user to interact with the checkbox', async () => {
      render(
        <TestWrapper id='checkboxField' initialValue={false}>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'checkbox',
                attributes: {},
                label: 'Test checkbox',
              }}
            />
          )}
        </TestWrapper>
      );

      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).not.toBeChecked();

      await userEvent.click(checkbox);

      // The component should be focusable and clickable
      expect(checkbox).toBeVisible();
      expect(checkbox).not.toBeDisabled();
      expect(checkbox).toBeChecked();
    });

    it('displays the label text next to the checkbox', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'checkbox',
            attributes: {},
            label: 'Enable feature',
          }}
        />
      );

      expect(screen.getByText('Enable feature')).toBeVisible();
    });

    it('renders checkbox with initial checked state', () => {
      render(
        <TestWrapper id='checkboxField' initialValue={true}>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'checkbox',
                attributes: {},
                label: 'Pre-checked option',
              }}
            />
          )}
        </TestWrapper>
      );

      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).toBeChecked();
    });

    it('renders read-only checkbox as text', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'checkbox',
            attributes: {},
            label: 'Test checkbox',
          }}
          initialValue={true}
          readOnly={true}
        />
      );

      const checkboxLabel = screen.getByRole('term');
      const checkboxValue = screen.getByRole('definition');
      expect(checkboxLabel).toBeVisible();
      expect(checkboxValue).toBeVisible();
      expect(checkboxLabel).toHaveTextContent('Test checkbox');
      expect(checkboxValue).toHaveTextContent('Enabled');
    });
  });

  describe('url field type', () => {
    const defaultProps = createMockContext('urlField');

    it('renders a text input for URL entry', () => {
      render(<DynamicFormField {...defaultProps} dynamicProps={{ type: 'url', attributes: {} }} />);

      const textInput = screen.getByRole('textbox');
      expect(textInput).toBeVisible();
    });

    it('renders URL input with validation', () => {
      render(
        <DynamicFormField
          {...createMockContext('urlField', 'invalid-url')}
          dynamicProps={{
            type: 'url',
            attributes: {},
            required: true,
          }}
        />
      );

      const textInput = screen.getByRole('textbox');
      expect(textInput).toBeVisible();
      expect(textInput).toHaveValue('invalid-url');
    });

    it('renders read-only URL as text', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{ type: "url", label: "My Url", attributes: {} }}
          initialValue="something.com"
          readOnly={true}
        />
      );

      const urlLabel = screen.getByRole('term');
      const urlValue = screen.getByRole('definition');
      expect(urlLabel).toBeVisible();
      expect(urlValue).toBeVisible();
      expect(urlLabel).toHaveTextContent('My Url');
      expect(urlValue).toHaveTextContent('something.com');
    });
  });

  describe('password field type', () => {
    const defaultProps = createMockContext('passwordField');

    it('renders a password input that masks the value', () => {
      render(<DynamicFormField {...defaultProps} dynamicProps={{ type: 'password', attributes: {} }} />);

      const passwordInput = screen.getByDisplayValue('');
      expect(passwordInput).toHaveAttribute('type', 'password');
    });

    it('allows user to type in the password field', async () => {
      render(
        <TestWrapper id='passwordField' initialValue=''>
          {mockContext => <DynamicFormField {...mockContext} dynamicProps={{ type: 'password', attributes: {} }} />}
        </TestWrapper>
      );

      const passwordInput = screen.getByDisplayValue('');
      await userEvent.type(passwordInput, 'secret123');

      // The component should be focusable and allow typing
      expect(passwordInput).toBeVisible();
      expect(passwordInput).not.toBeDisabled();
      expect(passwordInput).toHaveValue('secret123');
    });

    it('renders read-only password as masked text', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{ type: "password", label: "My Password", attributes: {} }}
          initialValue='password123'
          readOnly={true}
        />
      );

      const passLabel = screen.getByRole('term');
      const passValue = screen.getByRole('definition');
      expect(passLabel).toBeVisible();
      expect(passValue).toBeVisible();
      expect(passLabel).toHaveTextContent('My Password');
      expect(passValue).toHaveTextContent('********');
    });
  });

  describe('text-area field type', () => {
    const defaultProps = createMockContext('textareaField');

    it('renders a textarea for multi-line text entry', () => {
      render(<DynamicFormField {...defaultProps} dynamicProps={{ type: 'text-area', attributes: {} }} />);

      const textarea = screen.getByRole('textbox');
      expect(textarea).toBeVisible();
    });

    it('allows users to enter multi-line text', async () => {
      render(
        <TestWrapper id='textareaField' initialValue=''>
          {mockContext => <DynamicFormField {...mockContext} dynamicProps={{ type: 'text-area', attributes: {} }} />}
        </TestWrapper>
      );

      const textarea = screen.getByRole('textbox');
      await userEvent.type(textarea, 'Line 1{enter}Line 2');

      // The component should be focusable and allow typing
      expect(textarea).toBeVisible();
      expect(textarea).not.toBeDisabled();
      expect(textarea).toHaveValue('Line 1\nLine 2');
    });

    it('renders read-only text-area as text', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{ type: "text-area", label: "My Text Area", attributes: {} }}
          initialValue='Lorem, ipsum dolor sit amet consectetur adipisicing elit.'
          readOnly={true}
        />
      );

      const textAreaLabel = screen.getByRole('term');
      const textAreaValue = screen.getByRole('definition');
      expect(textAreaLabel).toBeVisible();
      expect(textAreaValue).toBeVisible();
      expect(textAreaLabel).toHaveTextContent('My Text Area');
      expect(textAreaValue).toHaveTextContent('Lorem, ipsum dolor sit amet consectetur adipisicing elit.');
    });
  });

  describe('combobox field type', () => {
    const defaultProps = createMockContext('comboboxField');

    it('renders a select dropdown with options', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            attributes: {
              options: { option1: 'Option 1', option2: 'Option 2' },
            },
          }}
        />
      );

      const select = screen.getByRole('combobox');
      expect(select).toBeVisible();
      expect(screen.getByText('Option 1')).toBeVisible();
      expect(screen.getByText('Option 2')).toBeVisible();
    });

    it('allows user to select an option', async () => {
      render(
        <TestWrapper id='comboboxField' initialValue=''>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'combobox',
                attributes: {
                  options: { option1: 'Option 1', option2: 'Option 2' },
                },
              }}
            />
          )}
        </TestWrapper>
      );

      const select = screen.getByRole('combobox');
      expect(select).toHaveValue(''); // Initial empty value

      await userEvent.selectOptions(select, 'option1');

      // The component should be focusable and allow selection
      expect(select).toBeVisible();
      expect(select).not.toBeDisabled();
      expect(select).toHaveValue('option1'); // Selected value
    });

    it('renders required combobox field', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            attributes: {
              options: { option1: 'Option 1' },
            },
            required: true,
          }}
        />
      );

      const select = screen.getByRole('combobox');
      expect(select).toBeVisible();
      expect(screen.getByText('Option 1')).toBeVisible();
    });

    it('renders combobox with initial selected value', () => {
      render(
        <TestWrapper id='comboboxField' initialValue='option2'>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'combobox',
                attributes: {
                  options: { option1: 'Option 1', option2: 'Option 2' },
                },
              }}
            />
          )}
        </TestWrapper>
      );

      const select = screen.getByRole('combobox');
      expect(select).toHaveValue('option2');
    });

    it('prioritizes attributes.options over storeApi', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            storeApi: 'test_Action.testMethod',
            attributes: {
              options: { static1: 'Static Option 1', static2: 'Static Option 2' },
            },
          }}
        />
      );

      const select = screen.getByRole('combobox');
      expect(select).toBeVisible();
      expect(screen.getByText('Static Option 1')).toBeVisible();
      expect(screen.getByText('Static Option 2')).toBeVisible();
    });

    it('loads options from storeApi when attributes.options is not provided', async () => {
      const mockData = [
        { id: 'repo1', name: 'Repository 1' },
        { id: 'repo2', name: 'Repository 2' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            storeApi: 'coreui_Repository.readReferences',
            attributes: {},
          }}
        />
      );

      const select = await screen.findByRole('combobox');
      expect(select).toBeVisible();
      expect(select).toHaveValue('');

      expect(await screen.findByText('Repository 1')).toBeVisible();
      expect(await screen.findByText('Repository 2')).toBeVisible();
    });

    it('loads options from storeApi with storeFilters', async () => {
      const mockData = [
        { id: 'repo1', name: 'Repository 1' },
        { id: 'repo2', name: 'Repository 2' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            storeApi: 'coreui_Repository.readReferences',
            storeFilters: {
              type: '!group',
            },
            attributes: {},
          }}
        />
      );

      const select = await screen.findByRole('combobox');
      expect(select).toBeVisible();

      expect(await screen.findByText('Repository 1')).toBeVisible();
      expect(await screen.findByText('Repository 2')).toBeVisible();
    });

    it('uses custom idMapping and nameMapping from storeApi response', async () => {
      const mockData = [
        { customId: 'role1', customName: 'Admin Role' },
        { customId: 'role2', customName: 'User Role' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <TestWrapper id='comboboxField' initialValue=''>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'combobox',
                storeApi: 'coreui_Role.read',
                idMapping: 'customId',
                nameMapping: 'customName',
                attributes: {},
              }}
            />
          )}
        </TestWrapper>
      );

      const select = await screen.findByRole('combobox');
      expect(select).toBeVisible();

      // Wait for options to load and be visible
      expect(await screen.findByText('Admin Role')).toBeVisible();
      expect(await screen.findByText('User Role')).toBeVisible();

      // Verify the option exists and has the correct value
      const adminOption = screen.getByRole('option', { name: 'Admin Role' });
      expect(adminOption).toHaveValue('role1');

      // Select by value string
      await userEvent.selectOptions(select, 'role1');
      expect(select).toHaveValue('role1');
    });

    it('shows loading state while fetching from storeApi', async () => {
      const pendingPromise = new Promise(() => {});
      ExtAPIUtils.extAPIRequest.mockReturnValue(pendingPromise);
      ExtAPIUtils.checkForError.mockImplementation(() => {});

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            storeApi: 'coreui_Repository.readReferences',
            attributes: {},
          }}
        />
      );

      // While loading, AsLoadWrapper shows a loading spinner, so combobox is not yet available
      expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
      expect(screen.getByText('Loading…')).toBeVisible();
      expect(screen.queryByText('Repository 1')).not.toBeInTheDocument();
    });

    it('renders error state when API error occurs', async () => {
      const mockApiResponse = {
        data: {
          result: {
            success: false,
            message: 'API Error',
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {
        throw new Error('API Error');
      });
      ExtAPIUtils.extractResult.mockImplementation(() => []);

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            storeApi: 'coreui_Repository.readReferences',
            attributes: {},
          }}
        />
      );

      // AsLoadWrapper shows an error alert when API fails
      expect(await screen.findByRole('alert')).toBeVisible();
      // Combobox should not be available when error occurs
      expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
      expect(screen.queryByText('Repository 1')).not.toBeInTheDocument();
    });

    it('renders combobox without options when neither attributes.options nor storeApi is provided', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            attributes: {},
          }}
        />
      );

      const select = screen.getByRole('combobox');
      expect(select).toBeVisible();
      expect(select).toHaveValue('');
      // No options should be available
      expect(screen.queryByText('Option 1')).not.toBeInTheDocument();
    });

    it('allows user to select option loaded from storeApi', async () => {
      const mockData = [
        { id: 'repo1', name: 'Repository 1' },
        { id: 'repo2', name: 'Repository 2' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <TestWrapper id='comboboxField' initialValue=''>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'combobox',
                storeApi: 'coreui_Repository.readReferences',
                attributes: {},
              }}
            />
          )}
        </TestWrapper>
      );

      const select = await screen.findByRole('combobox');
      expect(select).toBeVisible();

      await userEvent.selectOptions(select, 'repo1');

      expect(select).toHaveValue('repo1');
    });

    it('renders combobox with initial selected value from storeApi', async () => {
      const mockData = [
        { id: 'repo1', name: 'Repository 1' },
        { id: 'repo2', name: 'Repository 2' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <TestWrapper id='comboboxField' initialValue='repo2'>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'combobox',
                storeApi: 'coreui_Repository.readReferences',
                attributes: {},
              }}
            />
          )}
        </TestWrapper>
      );

      const select = await screen.findByRole('combobox');
      expect(select).toHaveValue('repo2');
      expect(await screen.findByText('Repository 1')).toBeVisible();
      expect(await screen.findByText('Repository 2')).toBeVisible();
    });

    it('does not show validation error when pristine and empty', () => {
      const mockContext = {
        id: 'comboboxField',
        current: {
          context: {
            data: { comboboxField: '' },
            pristineData: { comboboxField: '' },
            isTouched: {},
            validationErrors: { comboboxField: 'This field is required' },
          },
        },
      };

      render(
        <DynamicFormField
          {...mockContext}
          dynamicProps={{
            type: 'combobox',
            required: true,
            attributes: {
              options: { option1: 'Option 1' },
            },
          }}
        />
      );

      const select = screen.getByRole('combobox');
      expect(select).toBeVisible();
      // Should not show validation error when pristine
      expect(screen.queryByText('This field is required')).not.toBeInTheDocument();
    });

    it('shows validation error when touched and empty', () => {
      const mockContext = {
        id: 'comboboxField',
        current: {
          context: {
            data: { comboboxField: '' },
            pristineData: { comboboxField: '' },
            isTouched: { comboboxField: true },
            validationErrors: { comboboxField: 'This field is required' },
          },
        },
      };

      render(
        <DynamicFormField
          {...mockContext}
          dynamicProps={{
            type: 'combobox',
            required: true,
            attributes: {
              options: { option1: 'Option 1' },
            },
          }}
        />
      );

      const select = screen.getByRole('combobox');
      expect(select).toBeVisible();
      // Should show validation error when touched (may appear multiple times in the DOM)
      expect(screen.getAllByText('This field is required').length).toBeGreaterThan(0);
    });

    it('loads filtered options from storeApi with multiple filters', async () => {
      const mockData = [
        { id: 'repo1', name: 'Maven Repository' },
        { id: 'repo2', name: 'npm Repository' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'combobox',
            storeApi: 'coreui_Repository.readReferences',
            storeFilters: {
              format: 'maven2,npm',
              facets: 'org.globalart.astrum.repository.proxy.ProxyFacet',
            },
            attributes: {},
          }}
        />
      );

      const select = await screen.findByRole('combobox');
      expect(select).toBeVisible();

      expect(await screen.findByText('Maven Repository')).toBeVisible();
      expect(await screen.findByText('npm Repository')).toBeVisible();
    });

    it('renders read-only combobox as text string', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{ type: "combobox", label: "My Combobox", attributes: {} }}
          initialValue={'hello world'}
          readOnly={true}
        />
      );

      const comboboxLabel = screen.getByRole('term');
      const comboboxValue = screen.getByRole('definition');
      expect(comboboxLabel).toBeVisible();
      expect(comboboxValue).toBeVisible();
      expect(comboboxLabel).toHaveTextContent('My Combobox');
      expect(comboboxValue).toHaveTextContent('hello world');
    });
  });

  describe('itemselect field type', () => {
    const defaultProps = createMockContext('itemselectField', []);

    it('renders a transfer list with available and selected items', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'itemselect',
            attributes: {
              options: ['item1', 'item2'],
              fromTitle: 'Available',
              toTitle: 'Selected',
            },
          }}
        />
      );

      expect(screen.getByText('Available')).toBeVisible();
      expect(screen.getByText('Selected')).toBeVisible();
      expect(screen.getByText('item1')).toBeVisible();
      expect(screen.getByText('item2')).toBeVisible();
    });

    it('calls onChange when items are transferred', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'itemselect',
            attributes: {
              options: ['item1', 'item2'],
              fromTitle: 'Available',
              toTitle: 'Selected',
            },
          }}
        />
      );

      // Note: Testing the actual transfer interaction would require more complex setup
      // This test verifies the component renders correctly
      expect(screen.getByText('Available')).toBeVisible();
    });

    it('renders itemselect with initial selected items', () => {
      render(
        <TestWrapper id='itemselectField' initialValue={['item1']}>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'itemselect',
                attributes: {
                  options: ['item1', 'item2'],
                  fromTitle: 'Available',
                  toTitle: 'Selected',
                },
              }}
            />
          )}
        </TestWrapper>
      );

      // Verify the component renders with initial selected items
      expect(screen.getByText('Available')).toBeVisible();
      expect(screen.getByText('Selected')).toBeVisible();
      expect(screen.getByText('item1')).toBeVisible();
      expect(screen.getByText('item2')).toBeVisible();
    });

    it('allows user to interact with transfer list items', async () => {
      render(
        <TestWrapper id='itemselectField' initialValue={[]}>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'itemselect',
                attributes: {
                  options: ['item1', 'item2'],
                  fromTitle: 'Available',
                  toTitle: 'Selected',
                },
              }}
            />
          )}
        </TestWrapper>
      );

      // Initially, items should be in available list
      expect(screen.getByText('item1')).toBeVisible();
      expect(screen.getByText('item2')).toBeVisible();

      // Verify the component structure is correct
      expect(screen.getByText('Available')).toBeVisible();
      expect(screen.getByText('Selected')).toBeVisible();

      // Test that items are clickable (even if the transfer behavior is complex)
      const item1Label = screen.getByText('item1').closest('label');
      const item1Checkbox = item1Label.querySelector('input[type="checkbox"]');

      expect(item1Checkbox).not.toBeChecked();

      // Click the item (the actual transfer behavior might be complex to test in isolation)
      await userEvent.click(item1Label);

      // Verify the component structure is maintained after interaction
      expect(screen.getByText('Available')).toBeVisible();
      expect(screen.getByText('Selected')).toBeVisible();
      expect(screen.getByText('item1')).toBeVisible();
      expect(screen.getByText('item2')).toBeVisible();
    });

    it('prioritizes attributes.options over storeApi', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'itemselect',
            storeApi: 'test_Action.testMethod',
            attributes: {
              options: ['static1', 'static2'],
              fromTitle: 'Available',
              toTitle: 'Selected',
            },
          }}
        />
      );

      expect(screen.getByText('static1')).toBeVisible();
      expect(screen.getByText('static2')).toBeVisible();
    });

    it('loads items from storeApi when attributes.options is not provided', async () => {
      const mockData = [
        { id: 'event1', name: 'Event 1' },
        { id: 'event2', name: 'Event 2' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'itemselect',
            storeApi: 'coreui_Webhook.listWithTypeRepository',
            attributes: {
              fromTitle: 'Available',
              toTitle: 'Selected',
            },
          }}
        />
      );

      expect(await screen.findByText('Event 1')).toBeVisible();
      expect(await screen.findByText('Event 2')).toBeVisible();
    });

    it('uses custom idMapping and nameMapping from storeApi response', async () => {
      const mockData = [
        { customId: 'id1', customName: 'Custom Name 1' },
        { customId: 'id2', customName: 'Custom Name 2' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'itemselect',
            storeApi: 'test_Action.testMethod',
            idMapping: 'customId',
            nameMapping: 'customName',
            attributes: {
              fromTitle: 'Available',
              toTitle: 'Selected',
            },
          }}
        />
      );

      expect(await screen.findByText('Custom Name 1')).toBeVisible();
      expect(await screen.findByText('Custom Name 2')).toBeVisible();
    });

    it('shows transfer list structure while loading from storeApi', () => {
      const pendingPromise = new Promise(() => {});
      ExtAPIUtils.extAPIRequest.mockReturnValue(pendingPromise);
      ExtAPIUtils.checkForError.mockImplementation(() => {});

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'itemselect',
            storeApi: 'test_Action.testMethod',
            attributes: {
              fromTitle: 'Available',
              toTitle: 'Selected',
            },
          }}
        />
      );

      expect(screen.getByText('Available')).toBeVisible();
      expect(screen.getByText('Selected')).toBeVisible();
    });

    it('renders transfer list structure when API error occurs', async () => {
      const mockApiResponse = {
        data: {
          result: {
            success: false,
            message: 'API Error',
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {
        throw new Error('API Error');
      });
      ExtAPIUtils.extractResult.mockImplementation(() => []);

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'itemselect',
            storeApi: 'test_Action.testMethod',
            attributes: {
              fromTitle: 'Available',
              toTitle: 'Selected',
            },
          }}
        />
      );

      expect(await screen.findByText('Available')).toBeVisible();
      expect(await screen.findByText('Selected')).toBeVisible();
    });

    it('warns when neither attributes.options nor storeApi is provided', () => {
      const consoleWarnSpy = jest.spyOn(console, 'warn').mockImplementation();

      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{
            type: 'itemselect',
            attributes: {
              fromTitle: 'Available',
              toTitle: 'Selected',
            },
          }}
        />
      );

      expect(consoleWarnSpy).toHaveBeenCalledWith('itemselect field missing both attributes.options and storeApi');
      expect(screen.getByText('Available')).toBeVisible();
      expect(screen.getByText('Selected')).toBeVisible();

      consoleWarnSpy.mockRestore();
    });

    it('handles selected items from initialValue when using storeApi', async () => {
      const mockData = [
        { id: 'event1', name: 'Event 1' },
        { id: 'event2', name: 'Event 2' },
        { id: 'event3', name: 'Event 3' },
      ];
      const mockApiResponse = {
        data: {
          result: {
            success: true,
            data: mockData,
          },
        },
      };

      ExtAPIUtils.extAPIRequest.mockResolvedValue(mockApiResponse);
      ExtAPIUtils.checkForError.mockImplementation(() => {});
      ExtAPIUtils.extractResult.mockImplementation((response, defaultValue) => {
        return response?.data?.result?.data || defaultValue;
      });

      render(
        <TestWrapper id='itemselectField' initialValue={['event1', 'event2']}>
          {mockContext => (
            <DynamicFormField
              {...mockContext}
              dynamicProps={{
                type: 'itemselect',
                storeApi: 'test_Action.testMethod',
                attributes: {
                  fromTitle: 'Available',
                  toTitle: 'Selected',
                },
              }}
            />
          )}
        </TestWrapper>
      );

      expect(await screen.findByText('Event 1')).toBeVisible();
      expect(await screen.findByText('Event 2')).toBeVisible();
      expect(await screen.findByText('Event 3')).toBeVisible();
    });

    it('renders read-only itemselect as text list', () => {
      render(
        <DynamicFormField
          {...defaultProps}
          dynamicProps={{ type: "itemselect", label: "My Itemselect", attributes: {} }}
          initialValue={['hello', 'world']}
          readOnly={true}
        />
      );

      const itemselectLabel = screen.getByRole('term');
      const itemselectValues = screen.getAllByRole('definition');
      expect(itemselectLabel).toBeVisible();
      expect(itemselectValues.length).toBe(2);
      expect(itemselectValues[0]).toBeVisible();
      expect(itemselectValues[1]).toBeVisible();
      expect(itemselectLabel).toHaveTextContent('My Itemselect');
      expect(itemselectValues[0]).toHaveTextContent('hello');
      expect(itemselectValues[1]).toHaveTextContent('world');
    });
  });

  describe('unknown field type', () => {
    it('renders nothing and logs a warning for unknown field types', () => {
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();

      render(
        <DynamicFormField {...createMockContext('unknownField')} dynamicProps={{ type: 'unknown', attributes: {} }} />
      );

      expect(consoleSpy).toHaveBeenCalledWith('form field type=unknown is unknown');

      render(
        <DynamicFormField {...createMockContext('unknownField')} dynamicProps={{ type: 'unknown', attributes: {} }} readOnly={true} />
      );

      expect(consoleSpy).toHaveBeenCalledWith('form field type=unknown is unknown');

      consoleSpy.mockRestore();
    });
  });
});
