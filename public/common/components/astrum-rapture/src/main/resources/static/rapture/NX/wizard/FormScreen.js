
/*global Ext, NX*/

/**
 * Wizard form screen.
 *
 * @since 3.0
 * @abstract
 */
Ext.define('NX.wizard.FormScreen', {
  extend: 'NX.wizard.Screen',
  alias: 'widget.nx-wizard-formscreen',

  /**
   * Returns the screen form.
   *
   * @return {Ext.form.Basic}
   */
  getForm: function() {
    return this.down('form').getForm();
  }
});
