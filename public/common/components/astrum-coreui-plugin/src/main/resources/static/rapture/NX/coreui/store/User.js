
/*global Ext, NX*/

/**
 * User store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.User', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.User',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_User.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  pageSize: 100,

  sorters: { property: 'userId', direction: 'ASC' }
});
