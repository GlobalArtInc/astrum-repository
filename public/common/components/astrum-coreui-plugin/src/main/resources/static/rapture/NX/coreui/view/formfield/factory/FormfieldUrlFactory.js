
/*global Ext*/

/**
 * 'url' factory.
 *
 * @since 3.2
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldUrlFactory', {
  singleton: true,
  alias: [
    'nx.formfield.factory.url'
  ],

  /**
   * Creates a nx-url field.
   * @param formField form field to create nx-url for
   * @returns {*} created textfield (never null)
   */
  create: function (formField) {
    var item = {
      xtype: 'nx-url',
      htmlDecode: true,
      fieldLabel: formField.label,
      itemCls: formField.required ? 'required-field' : '',
      helpText: formField.helpText,
      allowBlank: !formField.required,
      disabled: formField.disabled,
      readOnly: formField.readOnly,
      useTrustStore: null,
      regex: formField.regexValidation ? new RegExp(formField.regexValidation) : null
    };
    if (formField.initialValue) {
      item.value = formField.initialValue;
    }
    return item;
  }

});

