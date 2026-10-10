
/*global Ext, AS*/

/**
 * Save search filter window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.search.SaveSearchFilter', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-search-save',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      title: AS.I18n.get('Search_SaveSearchFilter_Title'),
      defaultFocus: 'name',

      settingsForm: {
        xtype: 'as-settingsform',
        items: [
          {
            xtype: 'textfield',
            name: 'name',
            itemId: 'name',
            fieldLabel: AS.I18n.get('Search_SaveSearchFilter_Name_FieldLabel')
          },
          {
            xtype: 'textfield',
            name: 'description',
            fieldLabel: AS.I18n.get('Search_SaveSearchFilter_Description_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    });

    this.callParent();
  }

});
