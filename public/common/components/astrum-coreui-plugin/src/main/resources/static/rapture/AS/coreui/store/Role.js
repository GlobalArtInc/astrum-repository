
/*global Ext, AS*/

/**
 * Role store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.Role', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Role',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'AS.direct.coreui_Role.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' },

  listeners: {
    load: function(store, records) {
      // loop all roles, and sort the inner privileges array
      for (var i = 0; i < records.length; i++) {
        var role = records[i].data;
        role.privileges = Ext.Array.sort(role.privileges);
      }
    }
  }

});
