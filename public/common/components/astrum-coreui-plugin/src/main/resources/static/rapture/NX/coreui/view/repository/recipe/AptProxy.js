
/**
 * Repository settings form for an apt repository
 *
 * @since 3.17
 */
Ext.define('NX.coreui.view.repository.recipe.AptProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-apt-proxy',
  requires: [
    'NX.coreui.view.repository.facet.AptProxyFacet',
    'NX.coreui.view.repository.facet.AptSigningFacet',
    'NX.coreui.view.repository.facet.ProxyFacet',
    'NX.coreui.view.repository.facet.RoutingRuleFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
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
      {xtype: 'nx-aptui-repository-aptproxy-facet'},
      {xtype: 'nx-aptui-repository-aptsigning-facet', allowBlank: true},
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet'}
    ];

    me.callParent();
  }
});
