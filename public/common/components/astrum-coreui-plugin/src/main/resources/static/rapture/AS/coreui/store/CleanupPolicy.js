
/*global Ext, AS*/

/**
 * Cleanup Policy Store.
 *
 * @since 3.14
 */
Ext.define('AS.coreui.store.CleanupPolicy', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.CleanupPolicy',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.cleanup_CleanupPolicy.readByFormat'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: [
    { property: 'name', direction: 'ASC'}
  ]
});
