
/*global Ext, AS*/

/**
 * Roles / source store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.RoleBySource', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramOrder: 'source',

    api: {
      read: 'AS.direct.coreui_Role.readFromSource'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
