
/*global Ext, AS*/

/**
 * LDAP Schema Template store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.LdapSchemaTemplate', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.LdapSchemaTemplate',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.ldap_LdapServer.readTemplates'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
