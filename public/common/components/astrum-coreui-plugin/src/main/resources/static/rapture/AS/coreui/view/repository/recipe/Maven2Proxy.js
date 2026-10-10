
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Maven Proxy repository
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.Maven2Proxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-maven2-proxy',
  requires: [
    'AS.coreui.view.repository.facet.Maven2Facet',
    'AS.coreui.view.repository.facet.FirewallFacet',
    'AS.coreui.view.repository.facet.ProxyFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.RoutingRuleFacet',
    'AS.coreui.view.repository.facet.HttpClientFacetWithPreemptiveAuth',
    'AS.coreui.view.repository.facet.NegativeCacheFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-maven2-facet'},
      {xtype: 'as-coreui-repository-firewall-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet-with-preemptive-auth'}
    ];

    me.callParent();

    me.down('#remoteUrl').setHelpText(AS.I18n.get('Repository_Facet_ProxyFacet_Maven_Remote_HelpText'));
  }
});
