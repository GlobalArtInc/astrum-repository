
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Conan Proxy repository
 *
 * @since 3.28
 */
Ext.define('NX.coreui.view.repository.recipe.ConanProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-conan-proxy',
  requires: [
    'NX.coreui.view.repository.facet.ProxyFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.RoutingRuleFacet',
    'NX.coreui.view.repository.facet.HttpClientFacet',
    'NX.coreui.view.repository.facet.NegativeCacheFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet',
    'NX.coreui.view.repository.facet.ConanProxyFacet',
    'NX.coreui.view.repository.facet.FirewallFacet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-conanproxy-facet'},
      {xtype: 'nx-coreui-repository-firewall-facet'},
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet'}
    ];

    me.callParent();
    var conanVersionGroup = me.down('radiogroup[name=conanVersion]');
    var initialVersion = (conanVersionGroup && conanVersionGroup.getValue()
        && conanVersionGroup.getValue()['attributes.conan.conanVersion']) || 'V2';
    var helpTextKey = initialVersion === 'V1'
        ? 'Repository_Facet_ProxyFacet_Conan_V1_Remote_HelpText'
        : 'Repository_Facet_ProxyFacet_Conan_V2_Remote_HelpText';
    me.down('#remoteUrl').setHelpText(NX.I18n.get(helpTextKey));
  }
});
