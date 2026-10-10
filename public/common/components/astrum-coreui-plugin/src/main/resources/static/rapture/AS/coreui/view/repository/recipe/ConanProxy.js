
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Conan Proxy repository
 *
 * @since 3.28
 */
Ext.define('AS.coreui.view.repository.recipe.ConanProxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-conan-proxy',
  requires: [
    'AS.coreui.view.repository.facet.ProxyFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.RoutingRuleFacet',
    'AS.coreui.view.repository.facet.HttpClientFacet',
    'AS.coreui.view.repository.facet.NegativeCacheFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet',
    'AS.coreui.view.repository.facet.ConanProxyFacet',
    'AS.coreui.view.repository.facet.FirewallFacet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-conanproxy-facet'},
      {xtype: 'as-coreui-repository-firewall-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet'}
    ];

    me.callParent();
    var conanVersionGroup = me.down('radiogroup[name=conanVersion]');
    var initialVersion = (conanVersionGroup && conanVersionGroup.getValue()
        && conanVersionGroup.getValue()['attributes.conan.conanVersion']) || 'V2';
    var helpTextKey = initialVersion === 'V1'
        ? 'Repository_Facet_ProxyFacet_Conan_V1_Remote_HelpText'
        : 'Repository_Facet_ProxyFacet_Conan_V2_Remote_HelpText';
    me.down('#remoteUrl').setHelpText(AS.I18n.get(helpTextKey));
  }
});
