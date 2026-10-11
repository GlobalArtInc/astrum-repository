import { extractErrorMessage } from '../utils';

function axiosError(status: number, data: unknown): Error {
  const error = new Error(`Request failed with status code ${status}`) as Error & Record<string, unknown>;
  error.isAxiosError = true;
  error.response = { status, data, headers: {} };
  return error;
}

describe('extractErrorMessage', () => {
  it('shows the validation errors the server answers for a request', () => {
    const error = axiosError(400, [{ id: 'host', message: 'Host resolves to private/local IP address(es)' }]);

    expect(extractErrorMessage(error)).toBe('Host resolves to private/local IP address(es)');
  });

  it('shows the text of a server error instead of the status code', () => {
    const error = axiosError(500, 'Something broke on the server');

    expect(extractErrorMessage(error)).toContain('Something broke on the server');
    expect(extractErrorMessage(error)).not.toContain('status code 500');
  });

  it('keeps the message of other errors', () => {
    expect(extractErrorMessage(new Error('Plain failure'))).toBe('Plain failure');
    expect(extractErrorMessage('text')).toBe('text');
    expect(extractErrorMessage({ response: { data: { message: 'From data' } } })).toBe('From data');
  });
});
