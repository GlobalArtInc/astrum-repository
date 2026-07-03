

/*global Ext, NX*/

/*global NX*/

/**
 * 'staticInfo' factory.
 */
Ext.define('NX.coreui.view.formfield.factory.FormfieldStaticInfoFactory', {
  singleton: true,
  alias: [
      'nx.formfield.factory.staticInfo'
  ],

  create: function (formField) {
    const item = {
      xtype: 'container',
      layout: {
        type: 'vbox',
        align: 'stretch'
      },
      items: [
        {
          xtype: 'component',
          html: '<b>' + formField.label + '</b>'
        },
        {
          xtype: 'component',
          html: formField.helpText
        }
      ],
      validate: function () {},
    };
    return item;
  }
});
