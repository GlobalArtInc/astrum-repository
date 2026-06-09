
/*global Ext*/

/**
 * A hostname **{@link Ext.form.field.Text}**.
 *
 * @since 3.3
 */
Ext.define('NX.ext.form.field.Hostname', {
  extend: 'Ext.form.field.Text',
  requires: [
    'NX.util.Validator'
  ],
  alias: 'widget.nx-hostname',

  vtype: 'nx-hostname',
  maskRe: /\S/
});
