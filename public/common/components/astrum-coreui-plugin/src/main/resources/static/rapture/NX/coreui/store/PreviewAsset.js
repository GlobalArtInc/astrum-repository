
/*global Ext, NX*/

/**
 * Asset store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.PreviewAsset', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Asset',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Component.previewAssets'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  remote: true,
  autoLoad: false,

  buffered: true,
  pageSize: 50,

  remoteFilter: true,
  remoteSort: true,

  sorters: { property: 'name', direction: 'ASC' }

});
