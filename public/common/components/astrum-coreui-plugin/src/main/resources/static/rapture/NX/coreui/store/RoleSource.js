
/*global Ext, NX*/

/**
 * Role sources store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.RoleSource', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Reference',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.coreui_Role.readSources'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'id', direction: 'ASC' }
});
