
/*global Ext, AS*/

/**
 * @since 3.7
 */
Ext.define('AS.coreui.store.UploadComponentDefinition', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.UploadComponentDefinition',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Upload.getUploadDefinitions'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'format', direction: 'ASC' }
});
