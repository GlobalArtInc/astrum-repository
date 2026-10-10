
/*global Ext, AS*/

/**
 * Role grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.role.RoleList', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-role-list',
  requires: [
    'AS.Icons',
    'AS.I18n'
  ],

  stateful: true,
  stateId: 'as-coreui-role-list',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.store = 'Role';

    me.columns = [
      {
        xtype: 'as-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconName: function () {
          return 'role-default';
        }
      },
      {header: AS.I18n.get('Role_RoleList_Name_Header'), dataIndex: 'name', stateId: 'name', flex: 1},
      {header: AS.I18n.get('Role_RoleList_Source_Header'), dataIndex: 'source', stateId: 'source'},
      {header: AS.I18n.get('Role_RoleList_Description_Header'), dataIndex: 'description', stateId: 'description', flex: 1}
    ];

    me.viewConfig = {
      emptyText: AS.I18n.get('Role_RoleList_EmptyText'),
      deferEmptyText: false
    };

    me.plugins = [
      { ptype: 'gridfilterbox', emptyText: AS.I18n.get('Role_RoleList_Filter_EmptyText') }
    ];

    me.dockedItems = [{
      xtype: 'as-actions',
      items: [
        { xtype: 'button', text: AS.I18n.get('Role_RoleList_New_Button'), iconCls: 'x-fa fa-plus-circle', action: 'new', disabled: true,
          menu: [
            { text: AS.I18n.get('Role_RoleList_New_AstrumRoleItem'), action: 'newrole', iconCls: AS.Icons.cls('role-default', 'x16') }
          ]
        }
      ]
    }];

    me.callParent();
  }
});
