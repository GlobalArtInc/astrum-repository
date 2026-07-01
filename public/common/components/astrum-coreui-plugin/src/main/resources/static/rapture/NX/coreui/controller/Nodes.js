
/*global Ext, NX*/

/**
 * Nodes controller.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.controller.Nodes', {
  extend: 'NX.controller.Drilldown',
  requires: [
    'NX.view.info.Panel',
    'NX.view.info.Entry',
    'NX.util.Url',
    'NX.Permissions',
    'NX.I18n'
  ],
  masters: [
    'nx-coreui-system-nodelist'
  ],
  models: [
    'Node'
  ],
  stores: [
    'Node'
  ],
  views: [
    'system.Nodes',
    'system.NodeList',
    'system.NodeSettings',
    'system.NodeSettingsForm'
  ],
  refs: [
    {ref: 'feature', selector: 'nx-coreui-system-nodes'},
    {ref: 'list', selector: 'nx-coreui-system-nodelist'},
    {ref: 'info', selector: 'nx-coreui-system-nodes nx-coreui-system-node-settings'},
    {ref: 'content', selector: 'nx-feature-content'}
  ],

  features: {
    mode: 'admin',
    path: '/System/Nodes',
    text: 'Nodes',
    description: 'View nodes',
    view: 'NX.coreui.view.system.Nodes',
    iconConfig: {
      file: 'servers_network.png',
      variants: ['x16', 'x32']
    },
    visible: function () {
      return NX.Permissions.check('astrum:nodes:read') &&
        !NX.State.getValue('astrum.datastore.clustered.enabled');
    }
  },

  icons: {
    'node-default': {
      file: 'server.png',
      variants: ['x16', 'x32']
    }
  },

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.callParent();

    me.listen({
      controller: {
        '#Refresh': {
          refresh: me.loadStores
        }
      },
      store: {
        '#Node': {
          load: me.reselect
        }
      },
      component: {
        'nx-coreui-system-node-settings-form': {
          submitted: me.loadStores
        }
      }
    });
  },

  /**
   * @override
   */
  getDescription: function (model) {
    return model.get('friendlyName') || model.get('nodeIdentity');
  },

  onSelection: function(list, model) {
    if (Ext.isDefined(model)) {
      this.getInfo().loadRecord(model);
    }
  }
});
