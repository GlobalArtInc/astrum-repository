
/*global Ext, AS*/

/*global AS*/

/**
 * 'textfield' factory.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.formfield.factory.FormfieldTextFieldFactory', {
  singleton: true,
  alias: [
    'as.formfield.factory.textfield',
    'as.formfield.factory.string',
    'as.formfield.factory.password'
  ],

  /**
   * Creates a textfield.
   * @param formField form field to create textfield for
   * @returns {*} created textfield (never null)
   */
  create: function (formField) {
    var item = {
      xtype: formField.type === 'url' ? 'as-url' : 'textfield',
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
