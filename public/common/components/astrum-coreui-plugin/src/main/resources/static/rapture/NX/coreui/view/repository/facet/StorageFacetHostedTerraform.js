
/*global Ext, NX*/

/**
 * Configuration for Terraform repository storage write policy.
 * This facet excludes the Proprietary Components field as it is not applicable to Terraform format.
 */
Ext.define('NX.coreui.view.repository.facet.StorageFacetHostedTerraform', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-storage-hosted-terraform-facet',
  requires: [
    'NX.I18n'
  ],

  defaults: {
    allowBlank: false,
    itemCls: 'required-field'
  },

  /**
   * @cfg String
   * Set the write policy of storage, defaults to ALLOW_ONCE.
   */
  writePolicy: 'ALLOW_ONCE',

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        itemId: 'writePolicyFieldset',
        title: NX.I18n.get('Repository_Facet_StorageFacetHosted_Title'),

        items: [
          {
            xtype: 'combo',
            name: 'attributes.storage.writePolicy',
            itemId: 'writePolicy',
            fieldLabel: NX.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_HelpText'),
            emptyText: NX.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_EmptyText'),
            editable: false,
            store: [
              ['ALLOW', NX.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_AllowItem')],
              ['ALLOW_ONCE', NX.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_DisableItem')],
              ['DENY', NX.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_ReadOnlyItem')]
            ],
            value: me.writePolicy,
            queryMode: 'local'
          }
        ]
      }
    ];

    me.callParent();
  }

});