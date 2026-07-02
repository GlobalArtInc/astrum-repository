
/*global Ext, NX*/

/**
 * LDAP Server store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.LdapServer', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.LdapServer',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.ldap_LdapServer.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'order', direction: 'ASC' }
});
