
/*global Ext, AS*/

/**
 * Configuration specific to Http connections for repositories.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.facet.NegativeCacheFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-negativecache-facet',
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
        title: AS.I18n.get('Repository_Facet_NegativeCacheFacet_Title'),

        items: [
          {
            xtype: 'checkbox',
            name: 'attributes.negativeCache.enabled',
            fieldLabel: AS.I18n.get('Repository_Facet_NegativeCacheFacet_Enabled_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_NegativeCacheFacet_Enabled_HelpText'),
            value: true
          },
          {
            xtype: 'numberfield',
            name: 'attributes.negativeCache.timeToLive',
            fieldLabel: AS.I18n.get('Repository_Facet_NegativeCacheFacet_TTL_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_NegativeCacheFacet_TTL_HelpText'),
            minValue: -1,
            value: 1440
          }
        ]
      }
    ];

    me.callParent();
  }

});
