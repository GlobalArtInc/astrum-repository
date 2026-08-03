

import { interpret } from 'xstate';
import CapabilitiesEditMachine from './CapabilitiesEditMachine';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';
import Axios from 'axios';
import APIConstants from '../../../constants/APIConstants';

jest.mock('axios');

const { METHODS } = APIConstants.EXT.CAPABILITY;

describe('CapabilitiesEditMachine', () => {
  let mockExtAPIRequest;

  beforeEach(() => {
    jest.clearAllMocks();
    mockExtAPIRequest = jest.spyOn(ExtAPIUtils, 'extAPIRequest');
  });

  describe('load capability', () => {
    it('should start in loading state when initialized with an id', () => {
      mockSuccessfulApiCalls(buildMockCapabilities());

      const service = interpret(CapabilitiesEditMachine.withContext({ id: '789' }));
      service.start();

      expect(service.getSnapshot().value).toBe('loading');
      expect(service.getSnapshot().context.id).toBe('789');

      service.stop();
    });

    it('should have capability data in context on successful load', done => {
      mockSuccessfulApiCalls(buildMockCapabilities());

      const service = interpret(CapabilitiesEditMachine.withContext({ id: '123' })).onTransition(state => {
        if (state.matches('loaded')) {
          const expectedCapability = buildMockCapability();
          const expectedCapabilityType = buildMockCapabilityType(expectedCapability.typeId);
          // data should contain properties and enabled, not the full capability object
          const expectedData = {
            ...expectedCapability.properties,
            enabled: expectedCapability.enabled,
          };

          expect(state.context.capability).toEqual(expectedCapability);
          expect(state.context.capabilityType).toEqual(expectedCapabilityType);
          expect(state.context.data).toEqual(expectedData);
          expect(state.context.pristineData).toEqual(expectedData);
          done();
        }
      });

      service.start();
    });

    it('should transition to loadError state when capability is not found', done => {
      mockSuccessfulApiCalls(buildMockCapabilities());

      const service = interpret(CapabilitiesEditMachine.withContext({ id: 'non-existent' })).onTransition(state => {
        if (state.matches('loadError')) {
          expect(state.context.loadError).toBeDefined();
          done();
        }
      });

      service.start();
    });

    it('should reject when id is missing', done => {
      const service = interpret(CapabilitiesEditMachine.withContext({ id: null })).onTransition(state => {
        if (state.matches('loadError')) {
          expect(state.context.loadError).toContain('Missing capability id');
          done();
        }
      });

      service.start();
    });

    it('should transition to loadError state when API request fails', done => {
      const apiError = new Error('API request failed');
      mockExtAPIRequest.mockRejectedValue(apiError);

      const service = interpret(CapabilitiesEditMachine.withContext({ id: '123' })).onTransition(state => {
        if (state.matches('loadError')) {
          expect(state.context.loadError).toBeDefined();
          done();
        }
      });

      service.start();
    });

    it('should clear loadError on successful retry', done => {
      let attemptCount = 0;

      const capability = buildMockCapability();
      const capabilityType = buildMockCapabilityType(capability.typeId);

      mockExtAPIRequest.mockImplementation((action, method) => {
        // Handle Type Read Request
        if (method === METHODS.READ_TYPES) {
          return Promise.resolve({
            data: {
              result: {
                success: true,
                data: [capabilityType],
              },
            },
          });
        } else {
          // Handle Capability Read Request
          attemptCount++;
          if (attemptCount === 1) {
            // fail on first attempt to read capability
            return Promise.reject(new Error('Failed'));
          } else {
            // after first try succeed
            return Promise.resolve({
              data: {
                result: {
                  success: true,
                  data: [capability],
                },
              },
            });
          }
        }
      });

      const service = interpret(CapabilitiesEditMachine.withContext({ id: '123' })).onTransition(state => {
        if (state.matches('loadError')) {
          expect(state.context.loadError).toBeDefined();
          if (attemptCount > 1) {
            // we should not reach here, if we do check the mock implementation above
            done('failed to succeed on retry');
          }

          // First attempt failed, send retry
          service.send({ type: 'RETRY' });
        } else if (state.matches('loaded')) {
          // Retry successful
          expect(state.context.loadError).toBeNull();
          expect(state.context.capability).toEqual(capability);
          expect(state.context.capabilityType).toEqual(capabilityType);
          done();
        }
      });

      service.start();
    });
  });

  describe('delete capability', () => {
    it('should successfully delete capability and transition to ended state', done => {
      mockSuccessfulApiCalls(buildMockCapabilities());
      Axios.delete.mockResolvedValue({ data: { success: true } });

      let hasTriggeredDelete = false;
      let hasSeenAwaitingConfirmation = false;

      const statesTraversed = [];
      const service = interpret(CapabilitiesEditMachine.withContext({ id: '123' })).onTransition(state => {
        if (state.matches('loaded') && !hasTriggeredDelete) {
          // Once loaded (first time), send SHOW_DELETE_MODAL event
          hasTriggeredDelete = true;
          service.send({ type: 'SHOW_DELETE_MODAL' });
          statesTraversed.push('loaded');
        } else if (state.matches('awaitingDeleteConfirmation') && !hasSeenAwaitingConfirmation) {
          // After showing modal, send CONFIRM_DELETE event
          hasSeenAwaitingConfirmation = true;
          service.send({ type: 'CONFIRM_DELETE' });
          statesTraversed.push('awaitingDeleteConfirmation');
        } else if (state.matches('confirmDelete')) {
          // Verify we're in confirmDelete state
          expect(state.value).toBe('confirmDelete');
          statesTraversed.push('confirmDelete');
        } else if (state.matches('ended')) {
          // Verify successful deletion
          expect(state.value).toBe('ended');
          expect(state.context.deleteError).toBeNull();
          expect(Axios.delete).toHaveBeenCalledWith(expect.stringContaining('/capabilities/123'));
          service.stop();
          statesTraversed.push('ended');
          allStatesTraversedInOrder();
        }
      });

      service.start();

      function allStatesTraversedInOrder() {
        expect(statesTraversed).toEqual(['loaded', 'awaitingDeleteConfirmation', 'confirmDelete', 'ended']);
        done();
      }
    });

    it('should handle delete failure and transition to loaded state with error', done => {
      mockSuccessfulApiCalls(buildMockCapabilities());
      const deleteError = new Error('Failed to delete capability');
      Axios.delete.mockRejectedValue(deleteError);

      let hasSeenDeleteState = false;
      let hasSeenDeleteError = false;
      let hasSeenAwaitingConfirmation = false;

      const statesTraversed = [];
      const service = interpret(CapabilitiesEditMachine.withContext({ id: '123' })).onTransition(state => {
        if (state.matches('loaded') && !hasSeenDeleteState) {
          // Once loaded (first time), send SHOW_DELETE_MODAL event
          service.send({ type: 'SHOW_DELETE_MODAL' });
          statesTraversed.push('loaded');
        } else if (state.matches('awaitingDeleteConfirmation') && !hasSeenAwaitingConfirmation) {
          // After showing modal, send CONFIRM_DELETE event
          hasSeenAwaitingConfirmation = true;
          service.send({ type: 'CONFIRM_DELETE' });
          statesTraversed.push('awaitingDeleteConfirmation');
        } else if (state.matches('confirmDelete')) {
          // Verify we're in confirmDelete state
          hasSeenDeleteState = true;
          expect(state.value).toBe('confirmDelete');
          statesTraversed.push('confirmDelete');
        } else if (state.matches('loaded') && hasSeenDeleteState && !hasSeenDeleteError) {
          // After delete fails, we should be back in loaded state with error
          hasSeenDeleteError = true;
          expect(state.context.deleteError).toBeDefined();
          expect(state.context.deleteError).toContain('Failed to delete capability');
          expect(Axios.delete).toHaveBeenCalledWith(expect.stringContaining('/capabilities/123'));
          service.stop();

          statesTraversed.push('loaded-with-delete-state');
          allStatesTraversedInOrder();
        }
      });

      service.start();

      function allStatesTraversedInOrder() {
        expect(statesTraversed).toEqual([
          'loaded',
          'awaitingDeleteConfirmation',
          'confirmDelete',
          'loaded-with-delete-state',
        ]);
        done();
      }
    });
  });

  function buildMockCapabilities() {
    return [buildMockCapability(), { id: '456', typeId: 'other-capability', enabled: false }];
  }

  function buildMockCapability() {
    return {
      id: '123',
      typeId: 'test-capability',
      enabled: true,
      notes: 'Test notes',
      properties: {},
    };
  }

  function buildMockCapabilityType(id) {
    return {
      id: id,
      name: 'Test Type',
      about: 'Test capability type description',
      formFields: [],
    };
  }

  function mockSuccessfulApiCalls(capabilityResults) {
    // Generate capability types that match the capabilities being tested
    const capabilityTypes = capabilityResults.map(({ typeId }) => buildMockCapabilityType(typeId));

    // Mock both the READ call (for capabilities) and READ_TYPES call (for capability types)
    mockExtAPIRequest
      .mockResolvedValueOnce({
        data: {
          result: {
            success: true,
            data: capabilityResults,
          },
        },
      })
      .mockResolvedValueOnce({
        data: {
          result: {
            success: true,
            data: capabilityTypes,
          },
        },
      });
  }
});
