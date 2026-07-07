
/*global Ext, NX*/

/**
 * Repository "Settings" form for a NuGet Proxy repository.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.NugetProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-nuget-proxy',
  requires: [
    'NX.coreui.view.repository.facet.ProxyFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.RoutingRuleFacet',
    'NX.coreui.view.repository.facet.HttpClientFacet',
    'NX.coreui.view.repository.facet.NegativeCacheFacet',
    'NX.coreui.view.repository.facet.NugetProxyFacet',
    'NX.coreui.view.repository.facet.FirewallFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet',
    'NX.constants.FeatureFlags'
  ],

  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-nugetproxy-facet'},
      {xtype: 'nx-coreui-repository-firewall-facet'},
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet'}
    ];

    me.callParent();

    var nugetHelpTextKey = NX.State.getValue('nugetChocolateyEnabled') === true
        ? 'Repository_Facet_ProxyFacet_Nuget_Remote_HelpText_Chocolatey'
        : 'Repository_Facet_ProxyFacet_Nuget_Remote_HelpText';
    me.down('#remoteUrl').setHelpText(NX.I18n.get(nugetHelpTextKey));

    if (NX.State.getValue(NX.constants.FeatureFlags.NUGET_SYMBOL_SERVER_ENABLED)) {
      var container = me.down('#remoteUrl').ownerCt;
      var remoteUrlIndex = container.items.indexOf(me.down('#remoteUrl'));

      container.insert(remoteUrlIndex + 1,
          {
            xtype: 'nx-url',
            name: 'attributes.nugetProxy.symbolServerUrl',
            fieldLabel: NX.I18n.get('Repository_Facet_NugetProxyFacet_SymbolServerUrl_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_NugetProxyFacet_SymbolServerUrl_HelpText'),
            allowBlank: true,
            itemCls: ''
          }
      );

      container.insert(remoteUrlIndex + 2,
          {
            xtype: 'checkbox',
            name: 'attributes.nugetProxy.allowAnonymousSymbolAccess',
            fieldLabel: NX.I18n.get('Repository_Facet_NugetProxyFacet_AllowAnonymousSymbolAccess_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_NugetProxyFacet_AllowAnonymousSymbolAccess_HelpText'),
            value: true,
            itemCls: ''
          }
      );
    }
  }
});
