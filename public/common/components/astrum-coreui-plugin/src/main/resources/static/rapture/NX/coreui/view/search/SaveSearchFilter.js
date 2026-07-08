
/*global Ext, NX*/

/**
 * Save search filter window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.search.SaveSearchFilter', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-search-save',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      title: NX.I18n.get('Search_SaveSearchFilter_Title'),
      defaultFocus: 'name',

      settingsForm: {
        xtype: 'nx-settingsform',
        items: [
          {
            xtype: 'textfield',
            name: 'name',
            itemId: 'name',
            fieldLabel: NX.I18n.get('Search_SaveSearchFilter_Name_FieldLabel')
          },
          {
            xtype: 'textfield',
            name: 'description',
            fieldLabel: NX.I18n.get('Search_SaveSearchFilter_Description_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    });

    this.callParent();
  }

});
