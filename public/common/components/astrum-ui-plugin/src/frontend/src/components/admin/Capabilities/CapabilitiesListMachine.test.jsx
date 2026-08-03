
import { waitFor as waitForState } from 'xstate/lib/waitFor';
import APIConstants from '../../../constants/APIConstants';
import ExtAPIUtils from '../../../interface/ExtAPIUtils';
import TestUtils from '../../../interface/TestUtils';

import CapabilitiesListMachine from './CapabilitiesListMachine';
import { CAPABILITIES_TEST_DATA } from './Capabilities.testdata';

const {
  EXT: {
    CAPABILITY: { ACTION, METHODS },
  },
} = APIConstants;

// Mock ExtAPIUtils
jest.mock('../../../interface/ExtAPIUtils', () => ({
  extAPIRequest: jest.fn(),
  checkForErrorAndExtract: jest.fn(response => response.data),
}));

describe('CapabilitiesListMachine', () => {
  let mockExtAPIRequest;

  beforeEach(() => {
    jest.clearAllMocks();
    mockExtAPIRequest = ExtAPIUtils.extAPIRequest;
  });

  it('should have correct initial state and context', () => {
    const machine = CapabilitiesListMachine;
    const initialState = machine.initial;
    const initialContext = machine.context;

    expect(initialState).toBe('loading');
    expect(initialContext).toEqual({
      data: [],
      pristineData: [],
      sortField: 'typeName',
      sortDirection: 'asc',
      filter: '',
      error: '',
    });
  });

  it('should have correct machine id', () => {
    const machine = CapabilitiesListMachine;
    expect(machine.id).toBe('CapabilitiesListMachine');
  });

  it('should handle successful data fetch with real data', async () => {
    const testData = CAPABILITIES_TEST_DATA.REAL_DATA;
    const mockResponse = { data: testData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(mockExtAPIRequest).toHaveBeenCalledWith(ACTION, METHODS.READ);
      expect(finalState.context.data).toEqual(getExpectedSortedData(testData));
      expect(finalState.context.pristineData).toEqual(getExpectedPristineData(testData));
    });
  });

  it('should handle successful data fetch with synthetic data', async () => {
    const testData = CAPABILITIES_TEST_DATA.SAMPLE_DATA;
    const mockResponse = { data: testData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(mockExtAPIRequest).toHaveBeenCalledWith(ACTION, METHODS.READ);
      expect(finalState.context.data).toEqual(getExpectedSortedData(testData));
      expect(finalState.context.pristineData).toEqual(getExpectedPristineData(testData));
    });
  });

  it('should handle data fetch error', async () => {
    const errorMessage = 'Failed to fetch capabilities';
    const error = new Error(errorMessage);

    mockExtAPIRequest.mockRejectedValue(error);

    await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('error'));

      expect(mockExtAPIRequest).toHaveBeenCalledWith(ACTION, METHODS.READ);
      expect(finalState.context.error).toBeDefined();
    });
  });

  it('should handle empty data response', async () => {
    const emptyData = [];
    const mockResponse = { data: emptyData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(finalState.context.data).toEqual(emptyData);
      expect(finalState.context.pristineData).toEqual(emptyData);
    });
  });

  it('should handle large dataset', async () => {
    const largeData = CAPABILITIES_TEST_DATA.LARGE_DATASET;
    const mockResponse = { data: largeData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
      const finalState = await waitForState(service, state => state.matches('loaded'));

      expect(finalState.context.data).toEqual(getExpectedSortedData(largeData));
      expect(finalState.context.data).toHaveLength(50);
    });
  });

  it('should maintain state consistency during transitions', async () => {
    const testData = CAPABILITIES_TEST_DATA.SAMPLE_DATA;
    const mockResponse = { data: testData };

    mockExtAPIRequest.mockResolvedValue(mockResponse);

    await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
      const stateHistory = [];

      service.onTransition(state => {
        stateHistory.push({
          state: state.value,
          hasData: Array.isArray(state.context.data),
          dataLength: Array.isArray(state.context.data) ? state.context.data.length : 0,
          hasError: !!state.context.error,
        });
      });

      await waitForState(service, state => state.matches('loaded'));

      expect(stateHistory.length).toBeGreaterThan(0);
      expect(stateHistory[0].state).toEqual({ loading: 'fetch' });

      const lastState = stateHistory[stateHistory.length - 1];
      expect(lastState.state).toBe('loaded');
      expect(lastState.hasData).toBe(true);
      expect(lastState.dataLength).toBe(testData.length);
      expect(lastState.hasError).toBe(false);
    });
  });

  describe('filtering', () => {
    it('should filter by typeName', async () => {
      const testData = CAPABILITIES_TEST_DATA.REAL_DATA;
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter by "Audit" typeName
        service.send({ type: 'FILTER', filter: 'Audit' });

        await waitForState(service, state => state.context.data.length === 1);

        expect(service.state.context.data).toHaveLength(1);
        expect(service.state.context.data[0].typeName).toBe('Audit');
      });
    });

    it('should filter by description', async () => {
      const testData = CAPABILITIES_TEST_DATA.REAL_DATA;
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter by "Started" description
        service.send({ type: 'FILTER', filter: 'Started' });

        await waitForState(service, state => state.context.data.length === 1);

        expect(service.state.context.data).toHaveLength(1);
        expect(service.state.context.data[0].description).toBe('Started');
      });
    });

    it('should filter by notes', async () => {
      const testData = CAPABILITIES_TEST_DATA.REAL_DATA;
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter by "Automatically" in notes
        service.send({ type: 'FILTER', filter: 'Automatically' });

        await waitForState(service, state => state.context.data.length === 1);

        expect(service.state.context.data).toHaveLength(1);
        expect(service.state.context.data[0].notes).toContain('Automatically');
      });
    });

    it('should filter by tags.Category', async () => {
      const testData = CAPABILITIES_TEST_DATA.REAL_DATA;
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter by "Scheduling" category
        service.send({ type: 'FILTER', filter: 'Scheduling' });

        await waitForState(service, state => state.context.data.length === 1);

        expect(service.state.context.data).toHaveLength(1);
        expect(service.state.context.data[0].tags.Category).toBe('Scheduling');
      });
    });

    it('should filter by tags.Repository', async () => {
      const testData = [
        {
          ...CAPABILITIES_TEST_DATA.REAL_DATA[0],
          tags: {
            Category: 'Audit',
            Repository: 'maven-central',
          },
        },
        ...CAPABILITIES_TEST_DATA.REAL_DATA.slice(1),
      ];
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter by "maven-central" repository
        service.send({ type: 'FILTER', filter: 'maven-central' });

        await waitForState(service, state => state.context.data.length === 1);

        expect(service.state.context.data).toHaveLength(1);
        expect(service.state.context.data[0].tags.Repository).toBe('maven-central');
      });
    });

    it('should filter by state', async () => {
      const testData = CAPABILITIES_TEST_DATA.DATA_WITH_DIFFERENT_STATES;
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter by "passive" in state
        service.send({ type: 'FILTER', filter: 'pass' });

        await waitForState(service, state => state.context.data.length === 1);

        expect(service.state.context.data).toHaveLength(1);
        expect(service.state.context.data[0].state).toContain('passive');
      });
    });

    it('should be case insensitive', async () => {
      const testData = CAPABILITIES_TEST_DATA.REAL_DATA;
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter with lowercase
        service.send({ type: 'FILTER', filter: 'audit' });

        await waitForState(service, state => state.context.data.length === 1);

        expect(service.state.context.data).toHaveLength(1);
        expect(service.state.context.data[0].typeName).toBe('Audit');
      });
    });

    it('should return empty array when no matches found', async () => {
      const testData = CAPABILITIES_TEST_DATA.REAL_DATA;
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter with non-existent value
        service.send({ type: 'FILTER', filter: 'NonExistentCapability' });

        await waitForState(service, state => state.context.data.length === 0);

        expect(service.state.context.data).toHaveLength(0);
      });
    });

    it('should filter across multiple fields', async () => {
      const testData = CAPABILITIES_TEST_DATA.REAL_DATA;
      const mockResponse = { data: testData };

      mockExtAPIRequest.mockResolvedValue(mockResponse);

      await TestUtils.withTestMachine(CapabilitiesListMachine, async service => {
        await waitForState(service, state => state.matches('loaded'));

        // Filter by "Core" which appears in tags.Category
        service.send({ type: 'FILTER', filter: 'Core' });

        await waitForState(service, state => state.context.data.length === 1);

        expect(service.state.context.data).toHaveLength(1);
        expect(service.state.context.data[0].tags.Category).toBe('Core');
      });
    });
  });

  // new fields are added and data is sorted
  function getExpectedSortedData(data) {
    return getExpectedPristineData(data)
      .sort((a, b) => {
        const left = (a.typeName || '').toLowerCase();
        const right = (b.typeName || '').toLowerCase();

        if (left === right) {
          return 0;
        }

        return left > right ? 1 : -1;
      });
  }

  function getExpectedPristineData(data) {
    return data
        .map(item => ({
          ...item,
          category: item.tags?.Category || '',
          repository: item.tags?.Repository || ''
        }));
  }
});
