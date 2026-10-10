
/*global Ext, AS*/

/**
 * Wizard form screen.
 *
 * @since 3.0
 * @abstract
 */
Ext.define('AS.wizard.FormScreen', {
  extend: 'AS.wizard.Screen',
  alias: 'widget.as-wizard-formscreen',

  /**
   * Returns the screen form.
   *
   * @return {Ext.form.Basic}
   */
  getForm: function() {
    return this.down('form').getForm();
  }
});
