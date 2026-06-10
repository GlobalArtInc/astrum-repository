
/*global Ext*/

/**
 * Permission store.
 *
 * @since 3.0
 */
Ext.define('NX.store.Permission', {
  extend: 'Ext.data.Store',
  model: 'NX.model.Permission',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.rapture_Security.getPermissions'
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
