
/*global Ext, NX*/

/**
 * Task type store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.TaskType', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.TaskType',

  proxy: {
    type: 'direct',
    paramsAsHash: false,

    api: {
      read: 'NX.direct.coreui_Task.readTypes'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }
});
