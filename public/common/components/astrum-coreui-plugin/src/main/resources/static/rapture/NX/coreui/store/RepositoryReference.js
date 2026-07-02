
/*global Ext, NX*/

/**
 * Repository reference store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.RepositoryReference', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.RepositoryReference',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Repository.readReferences'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sortOnLoad: true,
  sorters: { property: 'name', direction: 'ASC' }

});
