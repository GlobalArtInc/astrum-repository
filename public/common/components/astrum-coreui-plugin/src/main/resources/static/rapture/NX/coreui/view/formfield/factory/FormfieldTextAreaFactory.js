
/*global Ext, NX*/

/*global NX*/

/**
 * 'textarea' factory.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldTextAreaFactory', {
  singleton: true,
  alias: [
    'nx.formfield.factory.textarea',
    'nx.formfield.factory.text-area'
  ],

  /**
   * Creates a textarea.
   * @param formField form field to create textarea for
   * @returns {*} created textarea (never null)
   */
  create: function (formField) {
    var item = {
      xtype: 'textarea',
      htmlDecode: true,
      width: '100%',
      fieldLabel: formField.label,
      itemCls: formField.required ? 'required-field' : '',
      helpText: formField.helpText,
      allowBlank: !formField.required,
      readOnly: formField.readOnly,
      regex: formField.regexValidation ? new RegExp(formField.regexValidation) : null,
      grow: true,
      growMax: 320
    };
    if (formField.initialValue) {
      item.value = formField.initialValue;
    }
    return item;
  }

});
