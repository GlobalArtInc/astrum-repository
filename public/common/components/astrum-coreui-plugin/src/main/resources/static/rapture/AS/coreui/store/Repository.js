
/*global Ext, AS*/

/**
 * Repository CMA format.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.Repository', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Repository',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Repository.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
