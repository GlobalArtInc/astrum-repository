
/*global Ext*/

/**
 * An **{@link Ext.form.field.Display}** that converts a date in ISO-8601 format to a date before display.
 *
 * @since 3.0
 */
Ext.define('NX.ext.form.field.DateDisplayField', {
  extend: 'Ext.form.field.Display',
  alias: 'widget.nx-datedisplayfield',

  config: {
    /**
     * @cfg {String} Format for Date output, defaults to ISO 8601.
     */
    format: 'c'
  },
  
  /**
   * @override
   */
  setValue: function (value) {
    if (value) {
      arguments[0] = Ext.Date.format(Ext.Date.parse(value, 'c'), this.format);
    }
    this.callParent(arguments);
  }

});
