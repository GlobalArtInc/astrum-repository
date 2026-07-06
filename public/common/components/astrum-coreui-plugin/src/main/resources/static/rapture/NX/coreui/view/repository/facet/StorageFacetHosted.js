
/*global Ext, NX*/

/**
 * Configuration for repository storage write policy.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.facet.StorageFacetHosted', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-storage-hosted-facet',
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
          },
          {
            xtype: 'checkbox',
            name: 'attributes.component.proprietaryComponents',
            itemId: 'proprietaryComponents',
            fieldLabel: NX.I18n.get('Repository_Facet_StorageFacetHosted_Proprietary_Components_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_StorageFacetHosted_Proprietary_Components_HelpText'),
            editable: true,
            value: false
          }
        ]
      }
    ];

    me.callParent();
  }

});
