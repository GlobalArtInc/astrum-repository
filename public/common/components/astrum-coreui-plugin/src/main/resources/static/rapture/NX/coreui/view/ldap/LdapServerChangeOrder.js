
/*global Ext, NX*/

/**
 * Change LDAP Server ordering window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerChangeOrder', {
  extend: 'NX.view.ChangeOrderWindow',
  alias: 'widget.nx-coreui-ldapserver-changeorder',
  requires: [
    'NX.coreui.store.LdapServer',
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = NX.I18n.get('Ldap_LdapServerChangeOrder_Title');

    me.store = Ext.create('NX.coreui.store.LdapServer', {
      sortOnLoad: true,
      sorters: { property: 'order', direction: 'ASC' }
    });
    me.store.load();

    me.callParent();
  }

});
