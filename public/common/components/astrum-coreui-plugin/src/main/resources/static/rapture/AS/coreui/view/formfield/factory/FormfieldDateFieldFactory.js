
/*global Ext, AS*/

/*global AS*/

/**
 * 'datefield' factory.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.formfield.factory.FormfieldDateFieldFactory', {
  singleton: true,
  alias: [
    'as.formfield.factory.datefield',
    'as.formfield.factory.date'
  ],

  /**
   * Creates a datefield.
   * @param formField form field to create datefield for
   * @returns {*} created datefield (never null)
   */
  create: function (formField) {
    var item = {
      xtype: 'datefield',
      htmlDecode: true,
      fieldLabel: formField.label,
      itemCls: formField.required ? 'required-field' : '',
      helpText: formField.helpText,
      allowBlank: !formField.required,
      regex: formField.regexValidation ? new RegExp(formField.regexValidation) : null,
      value: new Date()
    };
    if (formField.initialValue) {
      item.value = new Date(Number(formField.initialValue));
    }
    return item;
  }

});