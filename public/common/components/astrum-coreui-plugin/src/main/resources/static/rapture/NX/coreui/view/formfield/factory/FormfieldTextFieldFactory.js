
/*global Ext, NX*/

/*global NX*/

/**
 * 'textfield' factory.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldTextFieldFactory', {
  singleton: true,
  alias: [
    'nx.formfield.factory.textfield',
    'nx.formfield.factory.string',
    'nx.formfield.factory.password'
  ],

  /**
   * Creates a textfield.
   * @param formField form field to create textfield for
   * @returns {*} created textfield (never null)
   */
  create: function (formField) {
    var item = {
      xtype: formField.type === 'url' ? 'nx-url' : 'textfield',
      htmlDecode: true,
      fieldLabel: formField.label,
      itemCls: formField.required ? 'required-field' : '',
      helpText: formField.helpText,
      allowBlank: !formField.required,
      regex: formField.regexValidation ? new RegExp(formField.regexValidation) : null
    };
    if (formField.type === 'password') {
      item.inputType = 'password';
      item.inputAttrTpl = 'autocomplete="new-password"';
    }
    if (formField.initialValue) {
      item.value = formField.initialValue;
    }
    return item;
  }

});
