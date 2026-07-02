
/*global Ext, NX*/

/**
 * Repository CMA format.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.Repository', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Repository',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Repository.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
