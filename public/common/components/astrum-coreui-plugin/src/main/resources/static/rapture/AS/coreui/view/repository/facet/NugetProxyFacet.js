
/*global Ext, AS*/

/**
 * Configuration specific to Http connections for repositories.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.facet.NugetProxyFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-nugetproxy-facet',
  requires: [
    'AS.I18n'
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
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_NugetProxyFacet_Title'),

        items: [
          {
            xtype: 'radiogroup',
            name: 'nugetVersion',
            fieldLabel: AS.I18n.get('Repository_Facet_NugetProxyFacet_ProtocolVersion'),
            columns: 1,
            vertical: true,
            items: [
              {
                boxLabel: AS.I18n.get('Repository_Facet_NugetProxyFacet_V2'),
                name: 'attributes.nugetProxy.nugetVersion',
                inputValue: 'V2'
              },
              {
                boxLabel: AS.I18n.get('Repository_Facet_NugetProxyFacet_V3'),
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
            fieldLabel: AS.I18n.get('Repository_Facet_NugetProxyFacet_ItemMaxAge_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_NugetProxyFacet_ItemMaxAge_HelpText'),
            minValue: 0,
            value: 3600
          }
        ]
      }
    ];

    me.callParent();
  }

});
