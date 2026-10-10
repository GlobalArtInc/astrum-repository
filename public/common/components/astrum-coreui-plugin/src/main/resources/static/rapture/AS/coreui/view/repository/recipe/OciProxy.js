
/**
 * Repository "Settings" form for an OCI Proxy repository.
 *
 * @since 3.91
 */
Ext.define('AS.coreui.view.repository.recipe.OciProxy', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-oci-proxy',
  requires: [
    'AS.coreui.view.repository.facet.ProxyFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.oci.view.repository.facet.OciConnectorFacet',
    'AS.coreui.view.repository.facet.DockerProxyFacet',
    'AS.coreui.view.repository.facet.RoutingRuleFacet',
    'AS.coreui.view.repository.facet.HttpClientFacet',
    'AS.coreui.view.repository.facet.NegativeCacheFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-oci-repository-connector-facet'},
      {xtype: 'as-coreui-repository-docker-proxy-facet'},
      {xtype: 'as-coreui-repository-proxy-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-routing-rule-facet'},
      {xtype: 'as-coreui-repository-negativecache-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'},
      {xtype: 'as-coreui-repository-httpclient-facet'}
    ];

    me.callParent();

    // The OCI proxy backend (OciProxyRepositoryApiRequestToConfigurationConverter#normalizeRemoteUrl)
    // accepts oci:// and oci+http:// in addition to http/https — they're rewritten to https/http
    // on save. The shared 'as-url' vtype on the inherited remoteUrl field would reject those
    // schemes client-side before the backend ever sees them, so swap it for the OCI-specific
    // 'as-oci-url' vtype defined in AS.util.Validator.
    var remoteUrlField = me.down('#remoteUrl');
    if (remoteUrlField) {
      remoteUrlField.vtype = 'as-oci-url';
    }
  }
});
