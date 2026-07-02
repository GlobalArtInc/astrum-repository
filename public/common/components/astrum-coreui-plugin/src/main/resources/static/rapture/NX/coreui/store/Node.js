
/*global Ext, NX*/

/**
 * Node store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.Node', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Node',

  proxy: {
    type: 'direct',
    paramsAsHash: false,
    api: {
      read: 'NX.direct.proui_Node.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'local', direction: 'ASC' }

});
