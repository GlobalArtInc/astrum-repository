
/*global Ext, AS*/

/**
 * Date format related utils.
 *
 * @since 3.0
 */
Ext.define('AS.util.DateFormat', {
  singleton: true,

  mixins: [
    'AS.LogAware'
  ],

  /**
   * @private
   */
  defaultPatterns: {
    date: {
      // 2013-Mar-06
      'short': 'Y-M-d',

      // Wednesday, March 06, 2013
      'long': 'l, F d, Y'
    },

    time: {
      // 15:49:57
      'short': 'H:i:s',

      // 15:49:57-0700 PST (GMT-0700)
      'long': 'H:i:s T (\\G\\M\\TO)'
    },

    datetime: {
      // 2013-Mar-06 15:49:57
      'short': 'Y-M-d H:i:s',

      // Wednesday, March 06, 2013 15:50:19 PDT (GMT-0700)
      'long': 'l, F d, Y H:i:s T (\\G\\M\\TO)',

      // 2013/03/06 15:49:57 PDT (GMT-0700)
      'longVariant1': 'Y/m/d H:i:s T (\\G\\M\\TO)',
    }
  },

  /**
   * Return the date format object for the given name.
   *
   * Date format objects currently have a 'short' and 'long' variants.
   *
   *      @example
   *      var longDatetimePattern = AS.util.DateFormat.forName('datetime')['long'];
   *      var shortDatePattern = AS.util.DateFormat.forName('date')['short'];
   *
   * @public
   * @param name
   * @return {*} Date format object.
   */
  forName: function (name) {
    var format = this.defaultPatterns[name];

    // if no format, complain and return the full ISO-8601 format
    if (!name) {
      this.logWarn('Missing named format:', name);
      return 'c';
    }

    // TODO: Eventually let this customizable by user, for now its hardcoded

    return format;
  },

  /**
   * Formats the passed timestamp using the specified format pattern.
   *
   * @public
   * @param {Number} value The value to format converted to a date by the Javascript's built-in Date#parse method.
   * @param {String} [format] Any valid date format string. Defaults to {@link Ext.Date#defaultFormat}.
   * @return {String} The formatted date string
   */
  timestamp: function (value, format) {
    format = format || AS.util.DateFormat.forName('datetime')['long'];
    return value ? Ext.util.Format.date(new Date(value), format) : undefined;
  },

  /**
   * Returns a timestamp rendering function that can be reused to apply a date format multiple times efficiently.
   *
   * @public
   * @param {String} format Any valid date format string. Defaults to {@link Ext.Date#defaultFormat}.
   * @return {Function} The date formatting function
   */
  timestampRenderer: function (format) {
    return function (value) {
      return AS.util.DateFormat.timestamp(value, format);
    };
  },

  /**
   * @public
   * @returns {String} time zone
   */
  getTimeZone: function () {
    var me = this;

    if (!me.timeZone) {
      me.timeZone = new Date().toTimeString();
      me.timeZone = me.timeZone.substring(me.timeZone.indexOf(" "));
    }

    return me.timeZone;
  }

});
