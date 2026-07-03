
/*global Ext, NX*/

/*global NX*/

/**
 * 'checkbox' factory.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldCheckboxFactory', {
  singleton: true,
  alias: 'nx.formfield.factory.checkbox',

  /**
   * Creates a checkbox.
   * @param formField form field to create checkbox for
   * @returns {*} created checkbox (never null)
   */
  create: function (formField) {
    var item = {
      xtype: 'checkbox',
      fieldLabel: formField.label,
      helpText: formField.helpText,
      inputValue: true
    };
    if (formField.initialValue) {
      item.checked = Boolean('true' === formField.initialValue);
    }
    return item;
  }

});
