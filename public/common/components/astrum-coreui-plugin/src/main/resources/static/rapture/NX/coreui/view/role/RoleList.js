
/*global Ext, NX*/

/**
 * Role grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.role.RoleList', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-role-list',
  requires: [
    'NX.Icons',
    'NX.I18n'
  ],

  stateful: true,
  stateId: 'nx-coreui-role-list',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.store = 'Role';

    me.columns = [
      {
        xtype: 'nx-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconName: function () {
          return 'role-default';
        }
      },
      {header: NX.I18n.get('Role_RoleList_Name_Header'), dataIndex: 'name', stateId: 'name', flex: 1},
      {header: NX.I18n.get('Role_RoleList_Source_Header'), dataIndex: 'source', stateId: 'source'},
      {header: NX.I18n.get('Role_RoleList_Description_Header'), dataIndex: 'description', stateId: 'description', flex: 1}
    ];

    me.viewConfig = {
      emptyText: NX.I18n.get('Role_RoleList_EmptyText'),
      deferEmptyText: false
    };

    me.plugins = [
      { ptype: 'gridfilterbox', emptyText: NX.I18n.get('Role_RoleList_Filter_EmptyText') }
    ];

    me.dockedItems = [{
      xtype: 'nx-actions',
      items: [
        { xtype: 'button', text: NX.I18n.get('Role_RoleList_New_Button'), iconCls: 'x-fa fa-plus-circle', action: 'new', disabled: true,
          menu: [
            { text: NX.I18n.get('Role_RoleList_New_AstrumRoleItem'), action: 'newrole', iconCls: NX.Icons.cls('role-default', 'x16') }
          ]
        }
      ]
    }];

    me.callParent();
  }
});
