
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Maven Proxy repository
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.RawProxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-raw-proxy',
  requires: [
    'AS.coreui.view.repository.facet.RawFacet',
    'AS.coreui.view.repository.facet.FirewallFacet',
    'AS.coreui.view.repository.facet.RawProxyFacet',
    'AS.coreui.view.repository.facet.ProxyFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.RoutingRuleFacet',
    'AS.coreui.view.repository.facet.HttpClientFacet',
    'AS.coreui.view.repository.facet.NegativeCacheFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-raw-facet'},
      {xtype: 'as-coreui-repository-firewall-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet'}
    ];

    if (AS.State.getValue('rawQueryParamsForwardingEnabled')) {
      me.items.splice(1, 0, {xtype: 'as-coreui-repository-rawproxy-facet'});
    }

    me.callParent();
  }
});
