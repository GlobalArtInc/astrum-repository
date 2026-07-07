
/**
 * Repository "Settings" form for an Alpine Proxy repository
 */
Ext.define('NX.coreui.view.repository.recipe.AlpineProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-alpine-proxy',
  requires: [
    'NX.coreui.view.repository.facet.AlpineSigningFacet',
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
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-alpinesigning-facet'},
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'}
    ];

    me.callParent();
  }
});
