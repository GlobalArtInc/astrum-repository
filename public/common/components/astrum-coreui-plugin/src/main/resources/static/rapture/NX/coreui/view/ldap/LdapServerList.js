
/*global Ext, NX*/

/**
 * LDAP Server grid.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerList', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-ldapserver-list',
  requires: [
    'NX.I18n'
  ],

  stateful: true,
  stateId: 'nx-coreui-ldapserver-list',

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      store: 'LdapServer',

      columns: [
        {
          xtype: 'nx-iconcolumn',
          width: 36,
          iconVariant: 'x16',
          iconName: function () {
            return 'ldapserver-default';
          }
        },
        { header: NX.I18n.get('Ldap_LdapServerList_Order_Header'), dataIndex: 'order', stateId: 'order', width: 80 },
        { header: NX.I18n.get('Ldap_LdapServerList_Name_Header'), dataIndex: 'name', stateId: 'name', flex: 1, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Ldap_LdapServerList_URL_Header'), dataIndex: 'url', stateId: 'url', flex: 1, renderer: Ext.htmlEncode }
      ],

      viewConfig: {
        emptyText: NX.I18n.get('Ldap_LdapServerList_EmptyText'),
        deferEmptyText: false
      },

      dockedItems: [{
        xtype: 'nx-actions',
        items: [
          {
            xtype: 'button',
            text: NX.I18n.get('Ldap_LdapServerList_New_Button'),
            iconCls: 'x-fa fa-plus-circle',
            action: 'new',
            disabled: true
          },
          {
            xtype: 'button',
            text: NX.I18n.get('Ldap_LdapServerList_ChangeOrder_Button'),
            iconCls: 'x-fa fa-sort-numeric-up',
            action: 'changeorder',
            disabled: true
          },
          {
            xtype: 'button',
            text: NX.I18n.get('Ldap_LdapServerList_ClearCache_Button'),
            iconCls: 'x-fa fa-trash',
            action: 'clearcache',
            disabled: true
          }
        ]
      }],

      plugins: [
        { ptype: 'gridfilterbox', emptyText: NX.I18n.get('Ldap_LdapServerList_Filter_EmptyText') }
      ]
    });

    this.callParent();
  }

});
