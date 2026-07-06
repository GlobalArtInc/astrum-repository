
/*global Ext, NX*/

/**
 * Configuration specific to Http connections for repositories.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.facet.NugetProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-nugetproxy-facet',
  requires: [
    'NX.I18n'
  ],

  defaults: {
    itemCls: 'required-field'
  },

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_NugetProxyFacet_Title'),

        items: [
          {
            xtype: 'radiogroup',
            name: 'nugetVersion',
            fieldLabel: NX.I18n.get('Repository_Facet_NugetProxyFacet_ProtocolVersion'),
            columns: 1,
            vertical: true,
            items: [
              {
                boxLabel: NX.I18n.get('Repository_Facet_NugetProxyFacet_V2'),
                name: 'attributes.nugetProxy.nugetVersion',
                inputValue: 'V2'
              },
              {
                boxLabel: NX.I18n.get('Repository_Facet_NugetProxyFacet_V3'),
                name: 'attributes.nugetProxy.nugetVersion',
                inputValue: 'V3'
              }
            ],
            listeners: {
              afterrender: function(radioGroupForm) {
                if (Ext.Object.isEmpty(radioGroupForm.getValue())) {
                  var defaultValue = {
                    'attributes.nugetProxy.nugetVersion': 'V3'
                  };
                  radioGroupForm.setValue(defaultValue);
                }
              }
            }
          },
          {
            xtype: 'numberfield',
            name: 'attributes.nugetProxy.queryCacheItemMaxAge',
            fieldLabel: NX.I18n.get('Repository_Facet_NugetProxyFacet_ItemMaxAge_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_NugetProxyFacet_ItemMaxAge_HelpText'),
            minValue: 0,
            value: 3600
          }
        ]
      }
    ];

    me.callParent();
  }

});
