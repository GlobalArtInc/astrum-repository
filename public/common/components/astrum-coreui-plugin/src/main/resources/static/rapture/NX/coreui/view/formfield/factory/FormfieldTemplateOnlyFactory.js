
/*global Ext, NX*/

/*global NX*/

/**
 * 'templateOnly' factory that does not create UI components for template-only form fields.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldTemplateOnlyFactory', {
  singleton: true,
  alias: [
    'nx.formfield.factory.templateOnly',
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
