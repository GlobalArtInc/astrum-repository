
/*global Ext, AS*/

/*global AS*/

/**
 * 'templateOnly' factory that does not create UI components for template-only form fields.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.formfield.factory.FormfieldTemplateOnlyFactory', {
  singleton: true,
  alias: [
    'as.formfield.factory.templateOnly',
  ],

  /**
   * Returns null as Template-only form fields doesn't have UI representation.
   * Serves only as a placeholder for template generation.
   * @param formField form field instance
   * @returns {*} null always
   */
  create: function (formField) {
    return null;
  }

});
