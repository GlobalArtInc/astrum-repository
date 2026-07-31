

export class ApiUtils {
  static unwrapError(response) {
    if (response && response.data && response.data.error) {
      return response.data.error;
    } else if (response && response.data && response.data.message) {
      return response.data.message;
    } else if (response && response.statusText) {
      return response.statusText;
    } else if (response && response.status) {
      return `HTTP Error: ${response.status}`;
    } else {
      console.error('Unexpected response:', response);
      return 'Unknown error occurred';
    }
  }
}
