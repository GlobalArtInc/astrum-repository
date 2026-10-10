
/*global Ext, AS*/

/**
 * Repository reference store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.RepositoryReference', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.RepositoryReference',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Repository.readReferences'
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
