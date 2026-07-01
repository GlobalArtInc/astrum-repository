
/*global Ext, NX*/

/**
 * Component/Asset tree result store.
 *
 * @since 3.6
 */
(function() {
  Ext.define('NX.coreui.store.ComponentAssetTree', {
      extend: 'Ext.data.TreeStore',
      model: 'NX.coreui.model.ComponentAssetTree',
      autoLoad: false,
      paramOrder: ['node', 'repositoryName', 'filter'],
      defaultRootId: '/',
      folderSort: false,
      root: {
          node: '/',
          expanded: false
      },
      proxy: {
          type: 'direct',
          api: {
              read: 'NX.direct.coreui_Browse.read'
          },

          reader: {
              type: 'json',
              rootProperty: 'data',
              successProperty: 'success'
          }
      },
      listeners: {
          beforeload: function (store) {
              if (store.isLoading()) {
                  return false;
              }
          },
          nodeexpand: function(node) {
              if (node.childNodes && node.childNodes.length === 1) {
                  node.childNodes[0].expand();
              }
          }
      }
  });
}());
