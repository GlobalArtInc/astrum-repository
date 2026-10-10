
/*global Ext*/

/**
 * An email **{@link Ext.form.field.Text}**.
 *
 * @since 3.0
 */
Ext.define('AS.ext.form.field.Email', {
  extend: 'Ext.form.field.Text',
  alias: 'widget.as-email',
  requires: [
    'AS.util.Validator'
  ],

  vtype: 'as-email',
  maxLength: 254

});
