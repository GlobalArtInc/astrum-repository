
/*global Ext, AS*/

/**
 * Repository "Settings" form for a NuGet Proxy repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.NugetProxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-nuget-proxy',
  requires: [
    'AS.coreui.view.repository.facet.ProxyFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.RoutingRuleFacet',
    'AS.coreui.view.repository.facet.HttpClientFacet',
    'AS.coreui.view.repository.facet.NegativeCacheFacet',
    'AS.coreui.view.repository.facet.NugetProxyFacet',
    'AS.coreui.view.repository.facet.FirewallFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet',
    'AS.constants.FeatureFlags'
  ],

  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-nugetproxy-facet'},
      {xtype: 'as-coreui-repository-firewall-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet'}
    ];

    me.callParent();

    var nugetHelpTextKey = AS.State.getValue('nugetChocolateyEnabled') === true
        ? 'Repository_Facet_ProxyFacet_Nuget_Remote_HelpText_Chocolatey'
        : 'Repository_Facet_ProxyFacet_Nuget_Remote_HelpText';
    me.down('#remoteUrl').setHelpText(AS.I18n.get(nugetHelpTextKey));

    if (AS.State.getValue(AS.constants.FeatureFlags.NUGET_SYMBOL_SERVER_ENABLED)) {
      var container = me.down('#remoteUrl').ownerCt;
      var remoteUrlIndex = container.items.indexOf(me.down('#remoteUrl'));

      container.insert(remoteUrlIndex + 1,
          {
            xtype: 'as-url',
            name: 'attributes.nugetProxy.symbolServerUrl',
            fieldLabel: AS.I18n.get('Repository_Facet_NugetProxyFacet_SymbolServerUrl_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_NugetProxyFacet_SymbolServerUrl_HelpText'),
            allowBlank: true,
            itemCls: ''
          }
      );

      container.insert(remoteUrlIndex + 2,
          {
            xtype: 'checkbox',
            name: 'attributes.nugetProxy.allowAnonymousSymbolAccess',
            fieldLabel: AS.I18n.get('Repository_Facet_NugetProxyFacet_AllowAnonymousSymbolAccess_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_NugetProxyFacet_AllowAnonymousSymbolAccess_HelpText'),
            value: true,
            itemCls: ''
          }
      );
    }
  }
});
