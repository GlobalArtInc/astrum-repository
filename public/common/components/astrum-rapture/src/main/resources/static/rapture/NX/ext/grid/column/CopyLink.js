
/*global Ext, NX*/

/**
 * A {@link Ext.grid.column.Column} which renders its value as a link.
 *
 * @since 3.0
 */
Ext.define('NX.ext.grid.column.CopyLink', {
  extend: 'Ext.grid.column.Column',
  alias: ['widget.nx-copylinkcolumn'],
  requires: [
    'NX.util.Url'
  ],

  stateId: 'copylink',

  constructor: function () {
    var me = this;

    me.listeners = {
      click: function() {
        // Prevent drilldown from triggering
        return false;
      }
    };

    me.callParent(arguments);
  },

  /**
   * Renders value as a link.
   */
  defaultRenderer: function (value, _, record) {
    if (value) {
      value = value.replace(/\$baseUrl/, NX.util.Url.baseUrl);
      return NX.util.Url.asCopyWidget(value, record.data.format);
    }
    return undefined;
  },

  /**
   * @protected
   */
  target: function (value) {
    return value;
  }

});
