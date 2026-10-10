
/*global Ext*/

/**
 * A **{@link Ext.form.field.ComboBox}** with an extra button that allows value to be cleared.
 *
 * @since 3.0
 */
Ext.define('AS.ext.form.field.ClearableComboBox', {
  extend: 'Ext.form.field.ComboBox',
  alias: 'widget.as-clearablecombobox',

  triggers: {
    /**
     * Clear value.
     */
    clear: {
      cls: Ext.baseCSSPrefix + 'form-clear-trigger',
      handler: function() {
        this.reset();
      }
    }
  }

});
