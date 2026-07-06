
/*global Ext, NX*/

/**
 * Configuration for repository storage facet.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.facet.StorageFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-storage-facet',
  requires: [
    'NX.I18n'
  ],

  /**
   * @cfg Boolean
   * Set whether or not strict content type validation is enabled for storage, defaults to true.
   */
  strictContentTypeValidation: true,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;


    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_StorageFacet_Title'),

        defaults: {
          allowBlank: false,
          itemCls: 'required-field'
        },

        items: [
          {
            xtype: 'combo',
            name: 'attributes.storage.blobStoreName',
            itemId: 'blobStoreName',
            fieldLabel: NX.I18n.get('Repository_Facet_StorageFacet_BlobStore_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_StorageFacet_BlobStore_HelpText'),
            emptyText: NX.I18n.get('Repository_Facet_StorageFacet_BlobStore_EmptyText'),
            editable: false,
            store: 'BlobstoreNames',
            queryMode: 'local',
            displayField: 'name',
            valueField: 'name',
            readOnlyOnUpdate: true,
            listeners: {
              afterrender: function (combo) {
                if (!combo.getValue()) {
                  var store = combo.getStore();
                  if (store.getTotalCount() === 1) {
                    var value = store.getAt(0).get('name');
                    combo.originalValue = value;
                    combo.setValue(value);
                  }
                }
              }
            }
          },
          {
            xtype: 'checkbox',
            name: 'attributes.storage.strictContentTypeValidation',
            itemId: 'strictContentTypeValidation',
            fieldLabel: NX.I18n.get('Repository_Facet_StorageFacet_ContentTypeValidation_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_StorageFacet_ContentTypeValidation_HelpText'),
            value: me.strictContentTypeValidation
          }
        ]
      }
    ];

    if (NX.State.getValue('datastores')) {
      me.items[0].items.unshift(
          {
            xtype: 'hiddenfield',
            name: 'attributes.storage.dataStoreName',
            itemId: 'dataStoreName',
            editable: false,
            readOnlyOnUpdate: true,
            hidden: true,
            value: 'astrum'
          });
    }

    me.callParent();
  }
});
