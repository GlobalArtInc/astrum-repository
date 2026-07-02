
/*global Ext*/

/**
 * Privilege reference store.
 *
 * @since 3.1
 */
Ext.define('NX.coreui.store.PrivilegeReference', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Reference',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Privilege.readReferences'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },
  
  sorters: { property: 'name', direction: 'ASC' }
});
