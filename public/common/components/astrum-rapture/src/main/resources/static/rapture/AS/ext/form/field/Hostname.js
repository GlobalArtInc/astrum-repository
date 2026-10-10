
/*global Ext*/

/**
 * A hostname **{@link Ext.form.field.Text}**.
 *
 * @since 3.3
 */
Ext.define('AS.ext.form.field.Hostname', {
  extend: 'Ext.form.field.Text',
  requires: [
    'AS.util.Validator'
  ],
  alias: 'widget.as-hostname',

  vtype: 'as-hostname',
  maskRe: /\S/
});
