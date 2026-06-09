
/*global Ext, NX*/

/**
 * An URL **{@link Ext.form.field.Text}**.
 *
 * @since 3.0
 */
Ext.define('NX.ext.form.field.Url', {
  extend: 'Ext.form.field.Text',
  alias: 'widget.nx-url',
  requires: [
      'NX.util.Validator'
  ],
  vtype: 'nx-url',

  /**
   * @cfg {Boolean} trimOnBlur
   * True to automatically trim trailing/leading spaces when the field loses focus.
   * Defaults to true.
   */
  trimOnBlur: true,

  /**
   * @private
   */
  listeners: {
    blur: function() {
      if (this.trimOnBlur) {
        var value = this.getValue();
        if (value && typeof value === 'string') {
          this.setValue(value.trim());
        }
      }
    }
  },

  useTrustStore: function (field) {
    if (Ext.String.startsWith(field.getValue(), 'https://')) {
      return {
        name: 'useTrustStoreFor' + Ext.String.capitalize(field.getName()),
        url: field
      };
    }
    return undefined;
  }

});
