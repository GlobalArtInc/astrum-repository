
/*global Ext, AS*/

/**
 * Change LDAP Server ordering window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerChangeOrder', {
  extend: 'AS.view.ChangeOrderWindow',
  alias: 'widget.as-coreui-ldapserver-changeorder',
  requires: [
    'AS.coreui.store.LdapServer',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = AS.I18n.get('Ldap_LdapServerChangeOrder_Title');

    me.store = Ext.create('AS.coreui.store.LdapServer', {
      sortOnLoad: true,
      sorters: { property: 'order', direction: 'ASC' }
    });
    me.store.load();

    me.callParent();
  }

});
