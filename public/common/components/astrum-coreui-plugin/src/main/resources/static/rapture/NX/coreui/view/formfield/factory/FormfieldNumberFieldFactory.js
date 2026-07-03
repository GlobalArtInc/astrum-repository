
/*global Ext, NX*/

/*global NX*/

/**
 * 'numberfield' factory.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldNumberFieldFactory', {
  singleton: true,
  alias: [
    'nx.formfield.factory.numberfield',
    'nx.formfield.factory.number'
  ],

  /**
   * Creates a numberfield.
   * @param formField form field to create numberfield for
   * @returns {*} created numberfield (never null)
   */
  create: function (formField) {
    var item = {
      xtype: 'numberfield',
      fieldLabel: formField.label,
      itemCls: formField.required ? 'required-field' : '',
      helpText: formField.helpText,
      allowBlank: !formField.required,
      regex: formField.regexValidation ? new RegExp(formField.regexValidation) : null
    };
    if (formField.initialValue) {
      item.value = Number(formField.initialValue);
    }
    if(formField.minValue) {
      item.minValue = Number(formField.minValue);
    }
    if(formField.maxValue) {
      item.maxValue = Number(formField.maxValue);
    }
    return item;
  }

});
