
/*global Ext, AS*/

/**
 * Privilege grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.privilege.PrivilegeList', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-privilege-list',
  requires: [
    'AS.Icons',
    'AS.I18n'
  ],

  stateful: true,
  stateId: 'as-coreui-privilege-list',

  // filter will install into toolbar, ensure its properly styled for drilldown
  tbar: {
    xtype: 'as-actions'
  },

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    Ext.apply(me, {
      store: 'Privilege',

      columns: [
        {
          xtype: 'as-iconcolumn',
          dataIndex: 'type',
          width: 36,
          iconVariant: 'x16',
          iconNamePrefix: 'privilege-'
        },

        // NOTE: Not including ID here as for user-created privileges these are random strings

        { header: AS.I18n.get('Privilege_PrivilegeList_Name_Header'), dataIndex: 'name', stateId: 'name', flex: 2 },

        {
          header: AS.I18n.get('Privilege_PrivilegeList_Description_Header'),
          dataIndex: 'description',
          stateId: 'description',
          flex: 4
        },

        { header: AS.I18n.get('Privilege_PrivilegeList_Type_Header'), dataIndex: 'type', stateId: 'type', flex: 1 },

        {
          header: AS.I18n.get('Privilege_PrivilegeList_Permission_Header'),
          dataIndex: 'permission',
          stateId: 'permission',
          flex: 2
        }
      ],

      viewConfig: {
        emptyText: AS.I18n.get('Privilege_PrivilegeList_EmptyText'),
        emptyTextFilter: AS.I18n.get('Privilege_PrivilegeList_Filter_EmptyText'),
        deferEmptyText: false
      },

      selModel: {
        pruneRemoved: false
      },

      dockedItems: [{
        xtype: 'as-actions',
        items: [
          {
            xtype: 'button',
            text: AS.I18n.get('Privilege_PrivilegeList_New_Button'),
            action: 'new',
            disabled: true,
            iconCls: 'x-fa fa-plus-circle'
          }
        ]
      }],

      plugins: [
        {ptype: 'remotegridfilterbox'}
      ]
    });

    me.callParent();
  }
});
