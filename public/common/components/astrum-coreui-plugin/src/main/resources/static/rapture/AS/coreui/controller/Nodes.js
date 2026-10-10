
/*global Ext, AS*/

/**
 * Nodes controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.Nodes', {
  extend: 'AS.controller.Drilldown',
  requires: [
    'AS.view.info.Panel',
    'AS.view.info.Entry',
    'AS.util.Url',
    'AS.Permissions',
    'AS.I18n'
  ],
  masters: [
    'as-coreui-system-nodelist'
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
    {ref: 'feature', selector: 'as-coreui-system-nodes'},
    {ref: 'list', selector: 'as-coreui-system-nodelist'},
    {ref: 'info', selector: 'as-coreui-system-nodes as-coreui-system-node-settings'},
    {ref: 'content', selector: 'as-feature-content'}
  ],

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

    me.features = {
      mode: 'admin',
      path: '/System/Nodes',
      text: AS.I18n.get('Nodes_Text'),
      description: AS.I18n.get('Nodes_Description'),
      view: 'AS.coreui.view.system.Nodes',
      iconConfig: {
        file: 'servers_network.png',
        variants: ['x16', 'x32']
      },
      visible: function () {
        return AS.Permissions.check('astrum:nodes:read') &&
          !AS.State.getValue('astrum.datastore.clustered.enabled');
      }
    };

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
        'as-coreui-system-node-settings-form': {
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
