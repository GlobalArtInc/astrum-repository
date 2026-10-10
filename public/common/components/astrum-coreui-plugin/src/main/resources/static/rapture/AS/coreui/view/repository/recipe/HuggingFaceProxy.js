
/*global Ext, AS*/

/**
 * Repository "Settings" form for hugging face proxy content.
 *
 * @since 3.24
 */
Ext.define('AS.coreui.view.repository.recipe.HuggingFaceProxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-huggingface-proxy',
  requires: [
    'AS.coreui.view.repository.facet.FirewallFacet',
    'AS.coreui.view.repository.facet.ProxyFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.RoutingRuleFacet',
    'AS.coreui.view.repository.facet.BearerHttpClientFacet',
    'AS.coreui.view.repository.facet.NegativeCacheFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-firewall-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet-with-bearer-token'}
    ];
    me.callParent();
    Ext.ComponentQuery.query('checkbox[name=attributes.httpclient.autoBlock]')[0].setValue(false);

    me.down('#remoteUrl').setHelpText(AS.I18n.get('Repository_Facet_ProxyFacet_Huggingface_Remote_HelpText'));
  },

  listeners: {
    afterRender: function() {
      if (this.getRecord() === undefined) {
        const httpClientFacet = this.down('as-coreui-repository-httpclient-facet-with-bearer-token');
        httpClientFacet.down('numberfield[name=attributes.httpclient.connection.timeout]').setValue(120);
      }
    }
  }
});
