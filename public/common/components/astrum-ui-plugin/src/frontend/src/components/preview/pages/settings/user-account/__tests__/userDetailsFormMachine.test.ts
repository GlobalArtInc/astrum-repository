import { interpret } from 'xstate';
import { waitFor } from 'xstate/lib/waitFor';
import { createUserDetailsFormMachine, validateUserDetails } from '../userDetailsFormMachine';

jest.mock('../../../../../../interface/api', () => ({
  ENDPOINTS: {
    USER_ACCOUNT: '/service/rest/internal/ui/user',
  },
  restClient: {
    get: jest.fn(),
    put: jest.fn(),
  },
}));

const { restClient } = jest.requireMock('../../../../../../interface/api');

const MOCK_USER = {
  userId: 'jdoe',
  firstName: 'Jane',
  lastName: 'Doe',
  email: 'jane@example.com',
  external: false,
};

async function startAndLoad() {
  restClient.get.mockResolvedValue(MOCK_USER);
  const service = interpret(createUserDetailsFormMachine()).start();
  await waitFor(service, (state) => state.matches('editing'));
  return service;
}

describe('userDetailsFormMachine', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('loads the name and email of the signed-in user', async () => {
    const service = await startAndLoad();

    expect(service.getSnapshot().context.data).toEqual(MOCK_USER);
    service.stop();
  });

  it('saves the changed details to the account of the signed-in user', async () => {
    restClient.put.mockResolvedValue(undefined);
    const service = await startAndLoad();

    service.send({ type: 'UPDATE', name: 'lastName', value: ' Smith ' } as any);
    service.send({ type: 'UPDATE', name: 'email', value: 'jane.smith@example.com' } as any);
    service.send({ type: 'SUBMIT' } as any);

    await waitFor(service, (state) => state.matches('editing') && state.context.isPristine);

    expect(restClient.put).toHaveBeenCalledWith('/service/rest/internal/ui/user', {
      userId: 'jdoe',
      firstName: 'Jane',
      lastName: 'Smith',
      email: 'jane.smith@example.com',
      external: false,
    });
    service.stop();
  });

  it('does not save details that fail validation', async () => {
    const service = await startAndLoad();

    service.send({ type: 'UPDATE', name: 'email', value: 'not-an-email' } as any);
    service.send({ type: 'SUBMIT' } as any);

    expect(service.getSnapshot().matches('editing')).toBe(true);
    expect(restClient.put).not.toHaveBeenCalled();
    service.stop();
  });

  it('requires the names and a valid email', () => {
    expect(validateUserDetails(MOCK_USER)).toEqual({});
    expect(validateUserDetails({ ...MOCK_USER, firstName: ' ', lastName: '', email: '' })).toEqual({
      firstName: expect.any(String),
      lastName: expect.any(String),
      email: expect.any(String),
    });
    expect(validateUserDetails({ ...MOCK_USER, email: 'jane@' }).email).toMatch(/valid email/i);
  });
});
