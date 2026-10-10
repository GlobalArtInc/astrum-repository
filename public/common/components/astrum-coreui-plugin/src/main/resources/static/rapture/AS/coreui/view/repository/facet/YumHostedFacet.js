
/*global Ext, AS*/

/**
 * Configuration for Yum hosted repodata level.
 *
 * @since 3.8
 */
Ext.define('AS.coreui.view.repository.facet.YumHostedFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-yum-hosted-facet',
  requires: [
    'AS.I18n',
    'Ext.form.ComboBox'
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
        title: AS.I18n.get('Repository_Facet_YumHostedFacet_Title'),
        items: [
          {
            xtype: 'combo',
            name: 'attributes.yum.repodataDepth',
            fieldLabel: AS.I18n.get('Repository_Facet_YumHostedFacet_RepodataDepth_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_YumHostedFacet_RepodataDepth_HelpText'),
            forceSelection: true,
            editable: false,
            allowBlank: false,
            store : [0, 1, 2, 3, 4, 5]
          },
          {
            xtype: 'combo',
            name: 'attributes.yum.deployPolicy',
            fieldLabel: AS.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_HelpText'),
            emptyText: AS.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_EmptyText'),
            editable: false,
            store: [
              ['STRICT', AS.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_StrictItem')],
              ['PERMISSIVE', AS.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_PermissiveItem')]
            ],
            value: 'STRICT'
          }
        ]
      }
    ];

    me.callParent();
  }
});

