
/**
 * Repository "Settings" form for an OCI Proxy repository.
 *
 * @since 3.91
 */
Ext.define('NX.coreui.view.repository.recipe.OciProxy', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-oci-proxy',
  requires: [
    'NX.coreui.view.repository.facet.ProxyFacet',
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.oci.view.repository.facet.OciConnectorFacet',
    'NX.coreui.view.repository.facet.DockerProxyFacet',
    'NX.coreui.view.repository.facet.RoutingRuleFacet',
    'NX.coreui.view.repository.facet.HttpClientFacet',
    'NX.coreui.view.repository.facet.NegativeCacheFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-oci-repository-connector-facet'},
      {xtype: 'nx-coreui-repository-docker-proxy-facet'},
      {xtype: 'nx-coreui-repository-proxy-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-routing-rule-facet'},
      {xtype: 'nx-coreui-repository-negativecache-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
      {xtype: 'nx-coreui-repository-httpclient-facet'}
    ];

    me.callParent();

    // The OCI proxy backend (OciProxyRepositoryApiRequestToConfigurationConverter#normalizeRemoteUrl)
    // accepts oci:// and oci+http:// in addition to http/https — they're rewritten to https/http
    // on save. The shared 'nx-url' vtype on the inherited remoteUrl field would reject those
    // schemes client-side before the backend ever sees them, so swap it for the OCI-specific
    // 'nx-oci-url' vtype defined in NX.util.Validator.
    var remoteUrlField = me.down('#remoteUrl');
    if (remoteUrlField) {
      remoteUrlField.vtype = 'nx-oci-url';
    }
  }
});
