
/*global Ext, AS*/

/**
 * Node store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.Node', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Node',

  proxy: {
    type: 'direct',
    paramsAsHash: false,
    api: {
      read: 'AS.direct.proui_Node.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'local', direction: 'ASC' }

});
