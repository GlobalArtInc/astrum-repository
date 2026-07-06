
/*global Ext, NX*/

/**
 * Configuration specific to pypi proxy repos.
 */
Ext.define('NX.coreui.view.repository.facet.PyPiProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-pypi-proxy-facet',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_Pypi_Title'),
        items: [
          {
            xtype: 'textfield',
            name: 'attributes.pypi.indexPath',
            itemId: 'pypiIndexPath',
            fieldLabel: NX.I18n.get('Repository_Facet_Pypi_IndexPath_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_Pypi_IndexPath_HelpText'),
            allowBlank: true,
            value: '/simple',
            validator: function(value) {
              if (value === '/simple' || value === '') {
                return true;
              }
              return NX.I18n.get('Repository_Facet_Pypi_IndexPath_ValidationError');
            }
          }
        ]
      }
    ];

    me.callParent();
  }

});
