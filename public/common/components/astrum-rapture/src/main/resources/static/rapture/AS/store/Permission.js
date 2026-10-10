
/*global Ext*/

/**
 * Permission store.
 *
 * @since 3.0
 */
Ext.define('AS.store.Permission', {
  extend: 'Ext.data.Store',
  model: 'AS.model.Permission',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.rapture_Security.getPermissions'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sortOnLoad: true,
  sorters: { property: 'id', direction: 'ASC' }

});
