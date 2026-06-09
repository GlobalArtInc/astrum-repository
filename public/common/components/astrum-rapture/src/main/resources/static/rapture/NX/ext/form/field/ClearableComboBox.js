
/*global Ext*/

/**
 * A **{@link Ext.form.field.ComboBox}** with an extra button that allows value to be cleared.
 *
 * @since 3.0
 */
Ext.define('NX.ext.form.field.ClearableComboBox', {
  extend: 'Ext.form.field.ComboBox',
  alias: 'widget.nx-clearablecombobox',

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
