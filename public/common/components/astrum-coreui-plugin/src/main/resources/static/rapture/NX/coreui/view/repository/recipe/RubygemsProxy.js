
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Rubygems Proxy repository
 *
 * @since 3.1
 */
Ext.define('NX.coreui.view.repository.recipe.RubygemsProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-rubygems-proxy',
  requires: [
    'NX.coreui.view.repository.facet.FirewallFacet',
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
      {xtype: 'nx-coreui-repository-firewall-facet'},
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet'}
    ];

    me.callParent();

    me.down('#remoteUrl').setHelpText(NX.I18n.get('Repository_Facet_ProxyFacet_Rubygems_Remote_HelpText'));
  }
});
