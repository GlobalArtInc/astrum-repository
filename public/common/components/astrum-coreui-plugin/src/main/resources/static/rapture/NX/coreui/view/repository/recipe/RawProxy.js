
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Maven Proxy repository
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.RawProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-raw-proxy',
  requires: [
    'NX.coreui.view.repository.facet.RawFacet',
    'NX.coreui.view.repository.facet.FirewallFacet',
    'NX.coreui.view.repository.facet.RawProxyFacet',
    'NX.coreui.view.repository.facet.ProxyFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.RoutingRuleFacet',
    'NX.coreui.view.repository.facet.HttpClientFacet',
    'NX.coreui.view.repository.facet.NegativeCacheFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-raw-facet'},
      {xtype: 'nx-coreui-repository-firewall-facet'},
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet'}
    ];

    if (NX.State.getValue('rawQueryParamsForwardingEnabled')) {
      me.items.splice(1, 0, {xtype: 'nx-coreui-repository-rawproxy-facet'});
    }

    me.callParent();
  }
});
