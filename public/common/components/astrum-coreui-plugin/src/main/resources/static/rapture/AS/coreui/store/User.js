
/*global Ext, AS*/

/**
 * User store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.User', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.User',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_User.read'
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
