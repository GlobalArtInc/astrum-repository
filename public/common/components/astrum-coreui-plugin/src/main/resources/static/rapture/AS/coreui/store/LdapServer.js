
/*global Ext, AS*/

/**
 * LDAP Server store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.LdapServer', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.LdapServer',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.ldap_LdapServer.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'order', direction: 'ASC' }
});
