
/*global Ext, NX*/

/**
 * LDAP Schema Template store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.LdapSchemaTemplate', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.LdapSchemaTemplate',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.ldap_LdapServer.readTemplates'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
