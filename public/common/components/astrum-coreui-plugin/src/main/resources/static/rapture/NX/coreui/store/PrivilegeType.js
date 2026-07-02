
/*global Ext, NX*/

/**
 * Privilege type store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.PrivilegeType', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.PrivilegeType',
  requires: [
    'NX.State'
  ],

  proxy: {
    type: 'direct',
    paramsAsHash: false,
    api: {
      read: 'NX.direct.coreui_Privilege.readTypes'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
