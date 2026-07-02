
/*global Ext*/

/**
 * BrowseableFormats helper.
 *
 * @since 3.2.1
 */
Ext.define('NX.coreui.util.BrowseableFormats', {
  singleton: true,
  mixins: {
    logAware: 'NX.LogAware'
  },

  /**
   * Array of visible formats.
   *
   * @private
   */
  formats: undefined,

  /**
   * @public
   * @returns {boolean} True, if formats had been set (loaded from server)
   */
  available: function() {
    return Ext.isDefined(this.formats);
  },

  /**
   * Sets formats.
   *
   * @public
   * @param {Object} formats
   */
  setFormats: function(formats) {
    var me = this;

    // defensive copy
    me.formats = Ext.clone(formats);

    //<if debug>
    me.logDebug('BrowseableFormats installed');
    //</if>
  },

  /**
   * Resets all formats.
   *
   * @public
   */
  resetFormats: function() {
    var me = this;

    //<if debug>
    me.logDebug('Resetting formats');
    //</if>

    delete me.formats;
  },

  /**
   * Check if the format is in the current list of formats.
   *
   * @public
   * @param {String} format
   * @returns {boolean} True if format is in the list of visible formats.
   */
  check: function(format) {
    var me = this;

    if (!me.available()) {
      return false;
    }

    return Ext.Array.contains(me.formats, format);
  }
});
