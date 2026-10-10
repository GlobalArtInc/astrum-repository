
/*global Ext, AS*/

/**
 * Configuration for repository storage write policy.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.facet.StorageFacetHosted', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-storage-hosted-facet',
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
          },
          {
            xtype: 'checkbox',
            name: 'attributes.component.proprietaryComponents',
            itemId: 'proprietaryComponents',
            fieldLabel: AS.I18n.get('Repository_Facet_StorageFacetHosted_Proprietary_Components_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_StorageFacetHosted_Proprietary_Components_HelpText'),
            editable: true,
            value: false
          }
        ]
      }
    ];

    me.callParent();
  }

});
