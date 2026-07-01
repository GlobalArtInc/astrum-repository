
/*global Ext, NX*/

/**
 * Cleanup Policy Store.
 *
 * @since 3.14
 */
Ext.define('NX.coreui.store.CleanupPolicy', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.CleanupPolicy',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.cleanup_CleanupPolicy.readByFormat'
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
