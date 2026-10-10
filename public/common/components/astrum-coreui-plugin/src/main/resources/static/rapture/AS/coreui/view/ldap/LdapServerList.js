
/*global Ext, AS*/

/**
 * LDAP Server grid.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerList', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-ldapserver-list',
  requires: [
    'AS.I18n'
  ],

  stateful: true,
  stateId: 'as-coreui-ldapserver-list',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      store: 'LdapServer',

      columns: [
        {
          xtype: 'as-iconcolumn',
          width: 36,
          iconVariant: 'x16',
          iconName: function () {
            return 'ldapserver-default';
          }
        },
        { header: AS.I18n.get('Ldap_LdapServerList_Order_Header'), dataIndex: 'order', stateId: 'order', width: 80 },
        { header: AS.I18n.get('Ldap_LdapServerList_Name_Header'), dataIndex: 'name', stateId: 'name', flex: 1, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Ldap_LdapServerList_URL_Header'), dataIndex: 'url', stateId: 'url', flex: 1, renderer: Ext.htmlEncode }
      ],

      viewConfig: {
        emptyText: AS.I18n.get('Ldap_LdapServerList_EmptyText'),
        deferEmptyText: false
      },

      dockedItems: [{
        xtype: 'as-actions',
        items: [
          {
            xtype: 'button',
            text: AS.I18n.get('Ldap_LdapServerList_New_Button'),
            iconCls: 'x-fa fa-plus-circle',
            action: 'new',
            disabled: true
          },
          {
            xtype: 'button',
            text: AS.I18n.get('Ldap_LdapServerList_ChangeOrder_Button'),
            iconCls: 'x-fa fa-sort-numeric-up',
            action: 'changeorder',
            disabled: true
          },
          {
            xtype: 'button',
            text: AS.I18n.get('Ldap_LdapServerList_ClearCache_Button'),
            iconCls: 'x-fa fa-trash',
            action: 'clearcache',
            disabled: true
          }
        ]
      }],

      plugins: [
        { ptype: 'gridfilterbox', emptyText: AS.I18n.get('Ldap_LdapServerList_Filter_EmptyText') }
      ]
    });

    this.callParent();
  }

});
