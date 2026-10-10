
/**
 * Repository "Settings" form for an Alpine Proxy repository
 */
Ext.define('AS.coreui.view.repository.recipe.AlpineProxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-alpine-proxy',
  requires: [
    'AS.coreui.view.repository.facet.AlpineSigningFacet',
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
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-alpinesigning-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'}
    ];

    me.callParent();
  }
});
