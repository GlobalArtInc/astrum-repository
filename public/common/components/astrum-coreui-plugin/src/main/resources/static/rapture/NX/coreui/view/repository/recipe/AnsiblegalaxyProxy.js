
/*global Ext, NX*/

/**
 * Repository "Settings" form for an Ansible Galaxy Proxy repository
 */
Ext.define('NX.coreui.view.repository.recipe.AnsiblegalaxyProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-ansiblegalaxy-proxy',
  requires: [
    'NX.coreui.view.repository.facet.ProxyFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.HttpClientFacet',
    'NX.coreui.view.repository.facet.NegativeCacheFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet',
    'NX.coreui.view.repository.facet.RoutingRuleFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet'}
    ];

    me.callParent();

    me.down('#remoteUrl').setHelpText(NX.I18n.get('Repository_Facet_AnsibleGalaxyFacet_Remote_HelpText'));
  }
});
