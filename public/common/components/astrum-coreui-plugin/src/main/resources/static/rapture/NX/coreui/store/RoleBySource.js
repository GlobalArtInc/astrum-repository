
/*global Ext, NX*/

/**
 * Roles / source store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.RoleBySource', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramOrder: 'source',

    api: {
      read: 'NX.direct.coreui_Role.readFromSource'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
