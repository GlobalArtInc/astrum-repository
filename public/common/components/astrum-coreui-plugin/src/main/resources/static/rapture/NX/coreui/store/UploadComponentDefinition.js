
/*global Ext, NX*/

/**
 * @since 3.7
 */
Ext.define('NX.coreui.store.UploadComponentDefinition', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.UploadComponentDefinition',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Upload.getUploadDefinitions'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'format', direction: 'ASC' }
});
