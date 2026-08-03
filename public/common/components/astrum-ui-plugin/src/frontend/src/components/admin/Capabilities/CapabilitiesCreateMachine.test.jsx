

import { waitFor as waitForState } from 'xstate/lib/waitFor';
import APIConstants from '../../../constants/APIConstants';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';
import TestUtils from '../../../interface/TestUtils';

import CapabilitiesCreateMachine from './CapabilitiesCreateMachine';

const {
  EXT: {
    CAPABILITY: { ACTION, METHODS },
  },
} = APIConstants;

// Mock ExtAPIUtils
jest.mock('../../../interface/ExtAPIUtils', () => ({
  extAPIRequest: jest.fn(),
  checkForErrorAndExtract: jest.fn(response => response.data),
  checkForError: jest.fn(response => {
    const {data} = response;
    if (data?.type === 'exception') {
      throw Error(data.message);
    }
    if (data?.result && !data.result.success) {
      if (data.result.message) {
        throw Error(data.result.message);
      } else if (data.result.errors) {
        throw Error(JSON.stringify(data.result.errors));
      } else {
        throw Error('Unknown error');
      }
    }
  }),
  extractResult: jest.fn((response, defaultResult) => {
    const extDirectResponse = response?.data;
    return extDirectResponse?.result?.data || defaultResult;
  }),
}));

const CAPABILITY_TYPES = [
  {
    id: 'audit',
    name: 'Audit',
    about: '<p>Audit capability for tracking repository access and changes.</p>',
  },
  {
    id: 'baseurl',
    name: 'Base URL',
    about: '<p>Base URL configuration for the repository.</p>',
  },
  {
    id: 'scheduler',
    name: 'Scheduler',
    about: '<p>Scheduler capability for managing tasks and jobs.</p>',
  },
  {
    id: 'email',
    name: 'Email',
    about: '<p>Email configuration for notifications.</p>',
  },
];

describe('CapabilitiesCreateMachine', () => {
  let mockExtAPIRequest;

  beforeEach(() => {
    jest.clearAllMocks();
    mockExtAPIRequest = ExtAPIUtils.extAPIRequest;
  });

  it('should have correct initial state and context', () => {
    const machine = CapabilitiesCreateMachine;
    const initialState = machine.initial;
    const initialContext = machine.context;

    expect(initialState).toBe('loading');
    expect(initialContext.types).toEqual([]);
    expect(initialContext.loadError).toBeNull();
    expect(initialContext.data).toEqual({});
    expect(initialContext.pristineData).toEqual({});
    expect(initialContext.isPristine).toBe(true);
    expect(initialContext.isTouched).toEqual({});
    expect(initialContext.submitAttempted).toBe(false);
    expect(initialContext.shouldShowErrors).toBe(false);
  });

  it('should have correct machine id', () => {
    const machine = CapabilitiesCreateMachine;
    expect(machine.id).toBe('CapabilitiesCreateMachine');
  });

  it('should handle successful types fetch', async () => {
    const testData = CAPABILITY_TYPES;
    const mockResponse = { data: testData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(mockExtAPIRequest).toHaveBeenCalledWith(ACTION, METHODS.READ_TYPES);
      expect(finalState.context.types).toEqual(testData);
      expect(finalState.context.loadError).toBeNull();
    });
  });

  it('should handle types fetch error', async () => {
    const errorMessage = 'Failed to fetch capability types';
    const error = new Error(errorMessage);

    mockExtAPIRequest.mockRejectedValue(error);

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loadError'));

      expect(mockExtAPIRequest).toHaveBeenCalledWith(ACTION, METHODS.READ_TYPES);
      expect(finalState.context.loadError).toBe(errorMessage);
      expect(finalState.context.types).toEqual([]);
    });
  });

  it('should handle empty types response', async () => {
    const emptyData = [];
    const mockResponse = { data: emptyData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(finalState.context.types).toEqual(emptyData);
      expect(finalState.context.loadError).toBeNull();
    });
  });

  it('should handle unknown error messages', async () => {
    const error = new Error();

    mockExtAPIRequest.mockRejectedValue(error);

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loadError'));

      expect(finalState.context.loadError).toBeNull();
      expect(finalState.context.types).toEqual([]);
    });
  });

  it('should handle retry action from error state', async () => {
    const testData = CAPABILITY_TYPES;

    // First call fails
    mockExtAPIRequest.mockRejectedValueOnce(new Error('Failed to fetch'));
    // Second call succeeds
    mockExtAPIRequest.mockResolvedValueOnce({ data: testData });

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      // Wait for error state
      await waitForState(service, state => state.matches('loadError'));

      // Send retry action
      service.send('RETRY');

      // Wait for loaded state after retry
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(finalState.context.types).toEqual(testData);
      expect(finalState.context.loadError).toBeNull();
    });
  });

  it('should clear error on RETRY action', async () => {
    const error = new Error('Failed to fetch');

    // First call fails
    mockExtAPIRequest.mockRejectedValueOnce(error);
    // Second call succeeds
    mockExtAPIRequest.mockResolvedValueOnce({ data: CAPABILITY_TYPES });

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      // Wait for error state
      const errorState = await waitForState(service, state => state.matches('loadError'));
      expect(errorState.context.loadError).toBeDefined();
      expect(errorState.context.loadError).not.toBeNull();

      // Send retry action
      service.send('RETRY');

      // Wait for loaded state
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(finalState.context.loadError).toBeNull();
      expect(finalState.context.types).toEqual(CAPABILITY_TYPES);
    });
  });

  it('should maintain state consistency during transitions', async () => {
    const testData = CAPABILITY_TYPES;
    const mockResponse = { data: testData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      const stateHistory = [];

      service.onTransition(state => {
        stateHistory.push({
          state: state.value,
          hasTypes: Array.isArray(state.context.types),
          typesLength: Array.isArray(state.context.types) ? state.context.types.length : 0,
          hasError: !!state.context.loadError,
        });
      });

      await waitForState(service, state => state.matches('loaded'));

      expect(stateHistory.length).toBeGreaterThan(0);
      expect(stateHistory[0].state).toBe('loading');
      expect(stateHistory[0].hasTypes).toBe(true);
      expect(stateHistory[0].typesLength).toBe(0);
      expect(stateHistory[0].hasError).toBe(false);

      const lastState = stateHistory[stateHistory.length - 1];
      expect(lastState.state).toBe('loaded');
      expect(lastState.hasTypes).toBe(true);
      expect(lastState.typesLength).toBe(testData.length);
      expect(lastState.hasError).toBe(false);
    });
  });

  it('should handle error with message in event data', async () => {
    const errorMessage = 'Custom error message';
    const error = new Error(errorMessage);

    mockExtAPIRequest.mockRejectedValue(error);

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loadError'));

      expect(finalState.context.loadError).toBe(errorMessage);
      expect(finalState.context.types).toEqual([]);
    });
  });

  it('should transition through correct states', async () => {
    const testData = CAPABILITY_TYPES;
    const mockResponse = { data: testData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      const states = [];

      service.onTransition(state => {
        states.push(state.value);
      });

      await waitForState(service, state => state.matches('loaded'));

      // Should transition from loading to loaded
      expect(states).toContain('loading');
      expect(states).toContain('loaded');
      expect(states).not.toContain('loadError');
    });
  });

  it('should handle multiple capability types correctly', async () => {
    const manyTypes = [
      ...CAPABILITY_TYPES,
      { id: 'ldap', name: 'LDAP', about: '<p>LDAP configuration.</p>' },
      { id: 'saml', name: 'SAML', about: '<p>SAML configuration.</p>' },
      { id: 'ssl', name: 'SSL', about: '<p>SSL certificate management.</p>' },
    ];
    const mockResponse = { data: manyTypes };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(finalState.context.types).toEqual(manyTypes);
      expect(finalState.context.types).toHaveLength(7);
      expect(finalState.context.loadError).toBeNull();
    });
  });

  describe('shouldShowErrors state management', () => {
    const baseUrlType = {
      id: 'baseurl',
      name: 'Base URL',
      about: '<p>Base URL configuration for the repository.</p>',
      formFields: [
        {
          id: 'baseUrl',
          type: 'url',
          label: 'Base URL',
          helpText: 'The base URL for the repository.',
          required: true,
          disabled: false,
          readOnly: false,
          initialValue: null,
          attributes: {},
        },
      ],
    };

    it('should have shouldShowErrors as false initially', async () => {
      mockExtAPIRequest.mockResolvedValue({ data: [baseUrlType] });

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        const loadedState = await waitForState(service, state => state.matches('loaded'));

        expect(loadedState.context.shouldShowErrors).toBe(false);
        expect(loadedState.context.submitAttempted).toBe(false);
      });
    });

    it('should set shouldShowErrors to true when submit is attempted with invalid form', async () => {
      mockExtAPIRequest.mockResolvedValue({ data: [baseUrlType] });
      mockExtAPIRequest.mockResolvedValueOnce({ data: [baseUrlType] }); // For types fetch
      mockExtAPIRequest.mockResolvedValueOnce({ data: { success: true } }); // For save (won't be called due to validation)

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: baseUrlType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Attempt to save without filling required field (form is invalid)
        service.send({ type: 'SAVE' });

        const stateAfterSave = await waitForState(service, state => state.matches('loaded'));

        expect(stateAfterSave.context.submitAttempted).toBe(true);
        expect(stateAfterSave.context.shouldShowErrors).toBe(true);
        expect(Object.keys(stateAfterSave.context.validationErrors).length).toBeGreaterThan(0);
      });
    });

    it('should set shouldShowErrors to false when form becomes valid after submit attempt', async () => {
      mockExtAPIRequest.mockResolvedValue({ data: [baseUrlType] });
      mockExtAPIRequest.mockResolvedValueOnce({ data: [baseUrlType] }); // For types fetch
      mockExtAPIRequest.mockResolvedValueOnce({ data: { success: true } }); // For save

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: baseUrlType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Attempt to save without filling required field
        service.send({ type: 'SAVE' });
        await waitForState(service, state => state.matches('loaded'));

        let stateAfterInvalidSubmit = service.state;
        expect(stateAfterInvalidSubmit.context.shouldShowErrors).toBe(true);

        // Fill in the required field with valid URL
        service.send({
          type: 'UPDATE',
          data: { baseUrl: 'https://example.com' },
        });

        // Wait for the UPDATE transition to complete and validation to run
        // UPDATE transitions to 'loaded' which has entry actions including 'validate'
        const stateAfterUpdate = await waitForState(service, state => {
          // Wait for validation to run and clear errors
          // The validate action runs as an entry action when transitioning to 'loaded'
          return (
            state.matches('loaded') &&
            state.context.data?.baseUrl === 'https://example.com' &&
            Object.keys(state.context.validationErrors).length === 0
          );
        });

        // shouldShowErrors should be false because form is now valid (even though submitAttempted is true)
        expect(stateAfterUpdate.context.submitAttempted).toBe(true);
        expect(stateAfterUpdate.context.shouldShowErrors).toBe(false);
        expect(Object.keys(stateAfterUpdate.context.validationErrors).length).toBe(0);
      });
    });

    it('should reset shouldShowErrors to false when type changes', async () => {
      const auditType = {
        id: 'audit',
        name: 'Audit',
        about: '<p>Audit capability.</p>',
        formFields: [],
      };

      mockExtAPIRequest.mockResolvedValue({ data: [baseUrlType, auditType] });
      mockExtAPIRequest.mockResolvedValueOnce({ data: [baseUrlType, auditType] }); // For types fetch

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select baseurl type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: baseUrlType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Attempt to save without filling required field
        service.send({ type: 'SAVE' });
        await waitForState(service, state => state.matches('loaded'));

        let stateAfterInvalidSubmit = service.state;
        expect(stateAfterInvalidSubmit.context.shouldShowErrors).toBe(true);
        expect(stateAfterInvalidSubmit.context.submitAttempted).toBe(true);

        // Change type to audit (no form fields)
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: auditType,
        });

        const stateAfterTypeChange = await waitForState(service, state => state.matches('loaded'));

        // shouldShowErrors and submitAttempted should be reset
        expect(stateAfterTypeChange.context.shouldShowErrors).toBe(false);
        expect(stateAfterTypeChange.context.submitAttempted).toBe(false);
        expect(Object.keys(stateAfterTypeChange.context.validationErrors).length).toBe(0);
      });
    });
  });

  describe('save action', () => {
    const baseUrlType = {
      id: 'baseurl',
      name: 'Base URL',
      about: '<p>Base URL configuration for the repository.</p>',
      formFields: [
        {
          id: 'baseUrl',
          type: 'url',
          label: 'Base URL',
          helpText: 'The base URL for the repository.',
          required: true,
          disabled: false,
          readOnly: false,
          initialValue: null,
          attributes: {},
        },
      ],
    };

    const auditType = {
      id: 'audit',
      name: 'Audit',
      about: '<p>Audit capability for tracking repository access and changes.</p>',
      formFields: [],
    };

    const emailType = {
      id: 'email',
      name: 'Email',
      about: '<p>Email configuration for notifications.</p>',
      formFields: [
        {
          id: 'host',
          type: 'string',
          label: 'Host',
          helpText: 'The email server host.',
          required: true,
          disabled: false,
          readOnly: false,
          initialValue: null,
          attributes: {},
        },
        {
          id: 'port',
          type: 'number',
          label: 'Port',
          helpText: 'The email server port.',
          required: false,
          disabled: false,
          readOnly: false,
          initialValue: null,
          attributes: {},
        },
        {
          id: 'selectedRepositories',
          type: 'itemselect',
          label: 'Repositories',
          helpText: 'Select repositories.',
          required: true,
          disabled: false,
          readOnly: false,
          initialValue: null,
          attributes: {},
        },
        {
          id: 'notes',
          type: 'string',
          label: 'Notes',
          helpText: 'Additional notes.',
          required: false,
          disabled: false,
          readOnly: false,
          initialValue: null,
          attributes: {},
        },
      ],
    };

    it('should successfully save capability with valid form data', async () => {
      const mockSaveResponse = {
        data: {
          result: {
            success: true,
            data: [{ id: '123', typeId: 'baseurl', enabled: true }],
          },
        },
      };

      mockExtAPIRequest
        .mockResolvedValueOnce({ data: [baseUrlType] }) // For types fetch
        .mockResolvedValueOnce(mockSaveResponse); // For save

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: baseUrlType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Fill in required field
        service.send({
          type: 'UPDATE',
          data: { baseUrl: 'https://example.com' },
        });

        await waitForState(service, state => state.matches('loaded'));

        // Save
        service.send({ type: 'SAVE' });

        // Wait for saving state
        const savingState = await waitForState(service, state => state.matches('saving'));
        expect(savingState.value).toBe('saving');

        // Wait for save to complete (should transition back to loaded)
        const finalState = await waitForState(service, state => state.matches('loaded'));

        // Verify API was called correctly
        expect(mockExtAPIRequest).toHaveBeenCalledTimes(2);
        expect(mockExtAPIRequest).toHaveBeenNthCalledWith(1, ACTION, METHODS.READ_TYPES);
        expect(mockExtAPIRequest).toHaveBeenNthCalledWith(2, ACTION, METHODS.CREATE, {
          data: [
            {
              typeId: 'baseurl',
              enabled: true,
              notes: '',
              properties: {
                baseUrl: 'https://example.com',
              },
            },
          ],
        });

        // Verify no save errors
        expect(finalState.context.saveError).toBeUndefined();
        expect(finalState.context.saveErrors).toEqual({});
      });
    });

    it('should transform itemselect fields from array to comma-separated string in save payload', async () => {
      const mockSaveResponse = {
        data: {
          result: {
            success: true,
            data: [{ id: '123', typeId: 'email', enabled: true }],
          },
        },
      };

      mockExtAPIRequest
        .mockResolvedValueOnce({ data: [emailType] }) // For types fetch
        .mockResolvedValueOnce(mockSaveResponse); // For save

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: emailType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Fill in form fields including itemselect as array
        service.send({
          type: 'UPDATE',
          data: {
            host: 'smtp.example.com',
            port: 587,
            selectedRepositories: ['repo1', 'repo2', 'repo3'],
            notes: 'Test notes',
          },
        });

        await waitForState(service, state => state.matches('loaded'));

        // Save
        service.send({ type: 'SAVE' });

        // Wait for save to complete
        await waitForState(service, state => state.matches('loaded'));

        // Verify itemselect was transformed to comma-separated string
        expect(mockExtAPIRequest).toHaveBeenNthCalledWith(2, ACTION, METHODS.CREATE, {
          data: [
            {
              typeId: 'email',
              enabled: true,
              notes: 'Test notes',
              properties: {
                host: 'smtp.example.com',
                port: 587,
                selectedRepositories: 'repo1,repo2,repo3',
              },
            },
          ],
        });
      });
    });

    it('should remove notes from properties when notes is at top level', async () => {
      const mockSaveResponse = {
        data: {
          result: {
            success: true,
            data: [{ id: '123', typeId: 'email', enabled: true }],
          },
        },
      };

      mockExtAPIRequest
        .mockResolvedValueOnce({ data: [emailType] }) // For types fetch
        .mockResolvedValueOnce(mockSaveResponse); // For save

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: emailType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Fill in form fields (including notes in data)
        service.send({
          type: 'UPDATE',
          data: {
            host: 'smtp.example.com',
            selectedRepositories: ['repo1'],
            notes: 'Test notes',
          },
        });

        await waitForState(service, state => state.matches('loaded'));

        // Save
        service.send({ type: 'SAVE' });

        // Wait for save to complete
        await waitForState(service, state => state.matches('loaded'));

        // Verify notes is at top level and not in properties
        const saveCall = mockExtAPIRequest.mock.calls[1];
        const payload = saveCall[2].data[0];
        expect(payload.notes).toBe('Test notes');
        expect(payload.properties.notes).toBeUndefined();
      });
    });

    it('should save capability with no form fields (like audit)', async () => {
      const mockSaveResponse = {
        data: {
          result: {
            success: true,
            data: [{ id: '123', typeId: 'audit', enabled: true }],
          },
        },
      };

      mockExtAPIRequest
        .mockResolvedValueOnce({ data: [auditType] }) // For types fetch
        .mockResolvedValueOnce(mockSaveResponse); // For save

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: auditType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Save (no fields needed)
        service.send({ type: 'SAVE' });

        // Wait for saving state
        await waitForState(service, state => state.matches('saving'));

        // Wait for save to complete
        const finalState = await waitForState(service, state => state.matches('loaded'));

        // Verify API was called with empty properties
        expect(mockExtAPIRequest).toHaveBeenNthCalledWith(2, ACTION, METHODS.CREATE, {
          data: [
            {
              typeId: 'audit',
              enabled: true,
              notes: '',
              properties: {},
            },
          ],
        });

        expect(finalState.context.saveError).toBeUndefined();
      });
    });

    it('should handle save error and transition back to loaded state', async () => {
      const saveError = new Error('Failed to save capability');

      mockExtAPIRequest
        .mockResolvedValueOnce({ data: [baseUrlType] }) // For types fetch
        .mockRejectedValueOnce(saveError); // For save

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: baseUrlType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Fill in required field
        service.send({
          type: 'UPDATE',
          data: { baseUrl: 'https://example.com' },
        });

        await waitForState(service, state => state.matches('loaded'));

        // Save
        service.send({ type: 'SAVE' });

        // Wait for saving state
        await waitForState(service, state => state.matches('saving'));

        // Wait for error handling (should transition back to loaded)
        const finalState = await waitForState(service, state => {
          return state.matches('loaded');
        });

        // Verify we're back in loaded state after error
        expect(finalState.value).toBe('loaded');
        // Verify that saveErrorData is set (which indicates error handling occurred)
        // or that we at least transitioned back to loaded state (which confirms error was handled)
        expect(finalState.context.saveErrorData).toBeDefined();
      });
    });

    it('should handle API error response format', async () => {
      const mockErrorResponse = {
        data: {
          result: {
            success: false,
            message: 'Validation failed',
          },
        },
      };

      mockExtAPIRequest
        .mockResolvedValueOnce({ data: [baseUrlType] }) // For types fetch
        .mockResolvedValueOnce(mockErrorResponse); // For save (error response)

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: baseUrlType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Fill in required field
        service.send({
          type: 'UPDATE',
          data: { baseUrl: 'https://example.com' },
        });

        await waitForState(service, state => state.matches('loaded'));

        // Save
        service.send({ type: 'SAVE' });

        // Wait for error handling (checkForError will throw based on mock implementation)
        const finalState = await waitForState(service, state => {
          return state.matches('loaded') && state.context.saveError !== undefined;
        });

        expect(finalState.context.saveError).toBeDefined();
        expect(finalState.value).toBe('loaded');
      });
    });

    it('should transition through correct states during save', async () => {
      const mockSaveResponse = {
        data: {
          result: {
            success: true,
            data: [{ id: '123', typeId: 'baseurl', enabled: true }],
          },
        },
      };

      mockExtAPIRequest
        .mockResolvedValueOnce({ data: [baseUrlType] }) // For types fetch
        .mockResolvedValueOnce(mockSaveResponse); // For save

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        const statesTraversed = [];

        service.onTransition(state => {
          statesTraversed.push(state.value);
        });

        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: baseUrlType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Fill in required field
        service.send({
          type: 'UPDATE',
          data: { baseUrl: 'https://example.com' },
        });

        await waitForState(service, state => state.matches('loaded'));

        // Save
        service.send({ type: 'SAVE' });

        // Wait for save to complete
        await waitForState(service, state => state.matches('loaded'));

        // Verify state transitions
        expect(statesTraversed).toContain('loading');
        expect(statesTraversed).toContain('loaded');
        expect(statesTraversed).toContain('saving');
        // Should end back in loaded
        const lastState = statesTraversed[statesTraversed.length - 1];
        expect(lastState).toBe('loaded');
      });
    });

    it('should save capability with all optional fields', async () => {
      const optionalFieldsType = {
        id: 'optional',
        name: 'Optional Fields',
        about: '<p>Capability with all optional fields.</p>',
        formFields: [
          {
            id: 'field1',
            type: 'string',
            label: 'Field 1',
            required: false,
            disabled: false,
            readOnly: false,
            initialValue: null,
            attributes: {},
          },
          {
            id: 'field2',
            type: 'string',
            label: 'Field 2',
            required: false,
            disabled: false,
            readOnly: false,
            initialValue: null,
            attributes: {},
          },
        ],
      };

      const mockSaveResponse = {
        data: {
          result: {
            success: true,
            data: [{ id: '123', typeId: 'optional', enabled: true }],
          },
        },
      };

      mockExtAPIRequest
        .mockResolvedValueOnce({ data: [optionalFieldsType] }) // For types fetch
        .mockResolvedValueOnce(mockSaveResponse); // For save

      await TestUtils.withTestMachine(CapabilitiesCreateMachine, async service => {
        // Load types
        await waitForState(service, state => state.matches('loaded'));

        // Select type
        service.send({
          type: 'SET_SELECTED_TYPE',
          selectedType: optionalFieldsType,
        });

        await waitForState(service, state => state.matches('loaded'));

        // Save without filling any fields (all optional, pristine save allowed)
        service.send({ type: 'SAVE' });

        // Wait for saving state
        await waitForState(service, state => state.matches('saving'));

        // Wait for save to complete
        const finalState = await waitForState(service, state => state.matches('loaded'));

        // Verify save was successful
        expect(finalState.context.saveError).toBeUndefined();
        expect(mockExtAPIRequest).toHaveBeenNthCalledWith(2, ACTION, METHODS.CREATE, {
          data: [
            {
              typeId: 'optional',
              enabled: true,
              notes: '',
              properties: {},
            },
          ],
        });
      });
    });
  });
});
