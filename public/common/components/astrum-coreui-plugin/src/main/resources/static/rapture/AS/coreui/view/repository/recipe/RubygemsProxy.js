
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Rubygems Proxy repository
 *
 * @since 3.1
 */
Ext.define('AS.coreui.view.repository.recipe.RubygemsProxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-rubygems-proxy',
  requires: [
    'AS.coreui.view.repository.facet.FirewallFacet',
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
      {xtype: 'as-coreui-repository-firewall-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet'}
    ];

    me.callParent();

    me.down('#remoteUrl').setHelpText(AS.I18n.get('Repository_Facet_ProxyFacet_Rubygems_Remote_HelpText'));
  }
});
