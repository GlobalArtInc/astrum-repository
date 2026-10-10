
/*global Ext*/

/**
 * 'url' factory.
 *
 * @since 3.2
 */
Ext.define('AS.coreui.view.formfield.factory.FormfieldUrlFactory', {
  singleton: true,
  alias: [
    'as.formfield.factory.url'
  ],

  /**
   * Creates a as-url field.
   * @param formField form field to create as-url for
   * @returns {*} created textfield (never null)
   */
  create: function (formField) {
    var item = {
      xtype: 'as-url',
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

