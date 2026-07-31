

/**
 * @since 3.36
 *
 * Utilities for working with dates
 */

const ERROR_MESSAGE = value => `Unable to convert timestamp (${value}) to date string, returning null`;

export default class DateUtils {
  /**
   * @param {number} timestamp a numerical timestamp
   * @returns {string|null} a human-friendly date string
   */
  static timestampToString(timestamp) {
    try {
      return new Date(timestamp).toString();
    }
    catch {
      console.debug(ERROR_MESSAGE(timestamp));
      return null;
    }
  }

  /**
   * @param {number} timestamp a numerical timestamp
   * @returns {string|null} a "ddd, MMM D, YYYY" date format string. For example "Mon, Oct 3, 2022".
   */
  static prettyDate(timestamp) {
    try {
      const options = {
        weekday: 'short',
        month: 'short',
        year: 'numeric',
        day: 'numeric',
      };

      return new Date(timestamp).toLocaleDateString("en-US", options);
    }
    catch {
      console.debug(ERROR_MESSAGE(timestamp));
      return null;
    }
  }

  /**
   * @param {Date} date a Date
   * @returns {string|null} a human-friendly date time string
   */
  static prettyDateTime(date) {
    try {
      const options = {
        timeZoneName: 'longOffset',
        hour12: false,
      };
      return date.toLocaleString("en-US", options);
    }
    catch {
      console.debug(ERROR_MESSAGE(date));
      return null;
    }
  }

  /**
   * @param {Date} date a Date
   * @returns {string|null} a human-friendly date time string with long format
   */
  static prettyDateTimeLong(date) {
    try {
      const options = {
        weekday: 'long',
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false,
        timeZoneName: 'longOffset'
      };
      return date.toLocaleString("en-US", options);
    }
    catch {
      console.debug(ERROR_MESSAGE(date));
      return null;
    }
  }
}
