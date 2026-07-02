
/*global Ext, NX*/

/**
 * Task store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.Task', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Task',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.coreui_Task.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
