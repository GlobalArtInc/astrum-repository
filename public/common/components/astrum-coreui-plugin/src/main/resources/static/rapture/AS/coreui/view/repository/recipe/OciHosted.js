
/**
 * Repository "Settings" form for an OCI Hosted repository.
 *
 * @since 3.91
 */
Ext.define('AS.coreui.view.repository.recipe.OciHosted', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-oci-hosted',
  requires: [
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.StorageFacetHosted',
    'AS.oci.view.repository.facet.OciConnectorFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-oci-repository-connector-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-storage-hosted-facet', writePolicy: 'ALLOW'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'}
    ];

    me.callParent();
  }
});
