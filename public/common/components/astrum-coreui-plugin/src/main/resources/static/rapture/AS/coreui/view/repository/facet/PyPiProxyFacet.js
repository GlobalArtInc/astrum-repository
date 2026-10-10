
/*global Ext, AS*/

/**
 * Configuration specific to pypi proxy repos.
 */
Ext.define('AS.coreui.view.repository.facet.PyPiProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-pypi-proxy-facet',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_Pypi_Title'),
        items: [
          {
            xtype: 'textfield',
            name: 'attributes.pypi.indexPath',
            itemId: 'pypiIndexPath',
            fieldLabel: AS.I18n.get('Repository_Facet_Pypi_IndexPath_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_Pypi_IndexPath_HelpText'),
            allowBlank: true,
            value: '/simple',
            validator: function(value) {
              if (value === '/simple' || value === '') {
                return true;
              }
              return AS.I18n.get('Repository_Facet_Pypi_IndexPath_ValidationError');
            }
          }
        ]
      }
    ];

    me.callParent();
  }

});
