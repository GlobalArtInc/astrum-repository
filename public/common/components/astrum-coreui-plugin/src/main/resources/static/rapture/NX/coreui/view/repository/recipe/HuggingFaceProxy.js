
/*global Ext, NX*/

/**
 * Repository "Settings" form for hugging face proxy content.
 *
 * @since 3.24
 */
Ext.define('NX.coreui.view.repository.recipe.HuggingFaceProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-huggingface-proxy',
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
    Ext.ComponentQuery.query('checkbox[name=attributes.httpclient.autoBlock]')[0].setValue(false);

    me.down('#remoteUrl').setHelpText(NX.I18n.get('Repository_Facet_ProxyFacet_Huggingface_Remote_HelpText'));
  },

  listeners: {
    afterRender: function() {
      if (this.getRecord() === undefined) {
        const httpClientFacet = this.down('nx-coreui-repository-httpclient-facet-with-bearer-token');
        httpClientFacet.down('numberfield[name=attributes.httpclient.connection.timeout]').setValue(120);
      }
    }
  }
});
