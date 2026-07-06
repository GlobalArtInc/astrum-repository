
/*global Ext, NX*/

/**
 * Configuration for Yum hosted repodata level.
 *
 * @since 3.8
 */
Ext.define('NX.coreui.view.repository.facet.YumHostedFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-yum-hosted-facet',
  requires: [
    'NX.I18n',
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
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_YumHostedFacet_Title'),
        items: [
          {
            xtype: 'combo',
            name: 'attributes.yum.repodataDepth',
            fieldLabel: NX.I18n.get('Repository_Facet_YumHostedFacet_RepodataDepth_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_YumHostedFacet_RepodataDepth_HelpText'),
            forceSelection: true,
            editable: false,
            allowBlank: false,
            store : [0, 1, 2, 3, 4, 5]
          },
          {
            xtype: 'combo',
            name: 'attributes.yum.deployPolicy',
            fieldLabel: NX.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_HelpText'),
            emptyText: NX.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_EmptyText'),
            editable: false,
            store: [
              ['STRICT', NX.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_StrictItem')],
              ['PERMISSIVE', NX.I18n.get('Repository_Facet_YumHostedFacet_DeployPolicy_PermissiveItem')]
            ],
            value: 'STRICT'
          }
        ]
      }
    ];

    me.callParent();
  }
});

