
/*global Ext, AS*/

/**
 * Privilege type store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.PrivilegeType', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.PrivilegeType',
  requires: [
    'AS.State'
  ],

  proxy: {
    type: 'direct',
    paramsAsHash: false,
    api: {
      read: 'AS.direct.coreui_Privilege.readTypes'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
