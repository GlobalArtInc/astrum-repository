
/*global Ext*/

/**
 * An email **{@link Ext.form.field.Text}**.
 *
 * @since 3.0
 */
Ext.define('NX.ext.form.field.Email', {
  extend: 'Ext.form.field.Text',
  alias: 'widget.nx-email',
  requires: [
    'NX.util.Validator'
  ],

  vtype: 'nx-email',
  maxLength: 254

});
