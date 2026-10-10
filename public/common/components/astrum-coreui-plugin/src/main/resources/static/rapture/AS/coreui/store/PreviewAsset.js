
/*global Ext, AS*/

/**
 * Asset store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.PreviewAsset', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Asset',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Component.previewAssets'
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
