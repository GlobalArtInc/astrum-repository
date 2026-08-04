
import {ApiUtils} from './ApiUtils';

describe('ApiUtils', () => {
  describe('unwrapError', () => {
    it('handles object with message', () => {
      const response = { data: { message: 'Test error message' } };

      const result = ApiUtils.unwrapError(response);

      expect(result).toEqual('Test error message');
    });

    it('handles object with error', () => {
      const response = { data: { error: 'Error from server' } };

      const result = ApiUtils.unwrapError(response);

      expect(result).toEqual('Error from server');
    });

    it('handles object with statusText', () => {
      const response = { statusText: 'Not Found' };

      const result = ApiUtils.unwrapError(response);

      expect(result).toEqual('Not Found');
    });

    it('handles object with status', () => {
      const response = { status: 404 };

      const result = ApiUtils.unwrapError(response);

      expect(result).toEqual('HTTP Error: 404');
    });

    it('handles completely unexpected object', () => {
      // Mock console.error to suppress output in test
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => {});
      const response = { foo: 'bar' };

      const result = ApiUtils.unwrapError(response);

      expect(result).toEqual('Unknown error occurred');
      expect(consoleSpy).toHaveBeenCalledWith('Unexpected response:', response);
      consoleSpy.mockRestore();
    });
  });
});
