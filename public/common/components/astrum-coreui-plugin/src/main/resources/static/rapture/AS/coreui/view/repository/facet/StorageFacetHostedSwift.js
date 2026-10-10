
/*global Ext, AS*/

/**
 * Configuration for Swift repository storage write policy.
 * This facet excludes the Proprietary Components field as it is not applicable to Swift format.
 */
Ext.define('AS.coreui.view.repository.facet.StorageFacetHostedSwift', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-storage-hosted-swift-facet',
  requires: [
    'AS.I18n'
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
        cls: 'as-form-section',
        itemId: 'writePolicyFieldset',
        title: AS.I18n.get('Repository_Facet_StorageFacetHosted_Title'),

        items: [
          {
            xtype: 'combo',
            name: 'attributes.storage.writePolicy',
            itemId: 'writePolicy',
            fieldLabel: AS.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_HelpText'),
            emptyText: AS.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_EmptyText'),
            editable: false,
            store: [
              ['ALLOW', AS.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_AllowItem')],
              ['ALLOW_ONCE', AS.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_DisableItem')],
              ['DENY', AS.I18n.get('Repository_Facet_StorageFacetHosted_Deployment_ReadOnlyItem')]
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
