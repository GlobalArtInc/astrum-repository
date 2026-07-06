
/*global Ext, NX*/

/**
 * Configuration specific to Http connections for repositories.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.facet.NegativeCacheFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-negativecache-facet',
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
        title: NX.I18n.get('Repository_Facet_NegativeCacheFacet_Title'),

        items: [
          {
            xtype: 'checkbox',
            name: 'attributes.negativeCache.enabled',
            fieldLabel: NX.I18n.get('Repository_Facet_NegativeCacheFacet_Enabled_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_NegativeCacheFacet_Enabled_HelpText'),
            value: true
          },
          {
            xtype: 'numberfield',
            name: 'attributes.negativeCache.timeToLive',
            fieldLabel: NX.I18n.get('Repository_Facet_NegativeCacheFacet_TTL_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_NegativeCacheFacet_TTL_HelpText'),
            minValue: -1,
            value: 1440
          }
        ]
      }
    ];

    me.callParent();
  }

});
