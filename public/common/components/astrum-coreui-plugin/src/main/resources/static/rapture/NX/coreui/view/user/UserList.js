
/*global Ext, NX*/

/**
 * User grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.user.UserList', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-user-list',
  requires: [
    'NX.I18n'
  ],

  stateful: true,
  stateId: 'nx-coreui-user-list',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      store: 'User',

      columns: [
        {
          xtype: 'nx-iconcolumn',
          width: 36,
          iconVariant: 'x16',
          iconName: function() {
            return 'user-default';
          }
        },
        { header: NX.I18n.get('User_UserList_ID_Header'), dataIndex: 'userId', stateId: 'userId', flex: 2, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('User_UserList_Realm_Header'), dataIndex: 'realm', stateId: 'realm', renderer: Ext.htmlEncode },
        { header: NX.I18n.get('User_UserList_FirstName_Header'), dataIndex: 'firstName', stateId: 'firstName', flex: 2, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('User_UserList_LastName_Header'), dataIndex: 'lastName', stateId: 'lastName', flex: 2, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('User_UserList_Email_Header'), dataIndex: 'email', stateId: 'email', flex: 2, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('User_UserList_Status_Header'), dataIndex: 'status', stateId: 'status', renderer: Ext.htmlEncode }
      ],

      viewConfig: {
        emptyText: NX.I18n.get('User_UserList_EmptyText'),
        deferEmptyText: false
      },

      dockedItems: [{
        xtype: 'nx-actions',
        items: [
          {
            xtype: 'button',
            text: NX.I18n.get('User_UserList_New_Button'),
            iconCls: 'x-fa fa-plus-circle',
            action: 'new',
            disabled: true
          },
          '-',
          { xtype: 'label', text: NX.I18n.get('User_UserList_Source_Label') },
          { xtype: 'button', text: NX.I18n.get('User_UserList_Default_Button'), action: 'filter', menu: [] },
          { xtype: 'nx-coreui-user-searchbox' }
        ]
      }]
    });

    this.callParent();
  }

});
