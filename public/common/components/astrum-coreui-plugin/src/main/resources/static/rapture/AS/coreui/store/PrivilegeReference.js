
/*global Ext*/

/**
 * Privilege reference store.
 *
 * @since 3.1
 */
Ext.define('AS.coreui.store.PrivilegeReference', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Reference',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Privilege.readReferences'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },
  
  sorters: { property: 'name', direction: 'ASC' }
});
