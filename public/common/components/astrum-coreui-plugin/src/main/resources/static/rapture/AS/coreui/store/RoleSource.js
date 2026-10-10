
/*global Ext, AS*/

/**
 * Role sources store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.RoleSource', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.coreui_Role.readSources'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'id', direction: 'ASC' }
});
