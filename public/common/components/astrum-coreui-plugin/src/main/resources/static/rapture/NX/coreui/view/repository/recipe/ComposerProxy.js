
/*global Ext, NX*/

/**
 * Repository "Settings" form for a composer Proxy repository
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.ComposerProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-composer-proxy',
  requires: [
    'NX.coreui.view.repository.facet.FirewallFacet',
    'NX.coreui.view.repository.facet.ProxyFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.RoutingRuleFacet',
    'NX.coreui.view.repository.facet.BearerHttpClientFacet',
    'NX.coreui.view.repository.facet.NegativeCacheFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-firewall-facet'},
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet-with-bearer-token'}
    ];

    me.callParent();
    me.down('#remoteUrl').setHelpText(NX.I18n.get('Repository_Facet_ComposerFacet_Remote_HelpText'));
  }
});
