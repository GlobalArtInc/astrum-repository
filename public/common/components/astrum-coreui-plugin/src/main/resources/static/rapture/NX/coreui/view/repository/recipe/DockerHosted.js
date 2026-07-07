
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Docker Hosted repository.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.DockerHosted', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-docker-hosted',
  requires: [
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.StorageFacetHosted',
    'NX.coreui.view.repository.facet.DockerStorageFacetHosted',
    'NX.coreui.view.repository.facet.DockerConnectorFacet',
    'NX.coreui.view.repository.facet.DockerV1Facet',
    'NX.coreui.view.repository.facet.EvaluationFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-docker-connector-facet'},
      {xtype: 'nx-coreui-repository-docker-v1-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-docker-storage-hosted-facet', writePolicy: 'ALLOW'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'}
    ];

    // Only add evaluation facet if feature is enabled
    if (NX.State.getValue('hostedRepositoryEvaluationEnabled')) {
      me.items.splice(me.items.length - 1, 0, {xtype: 'nx-coreui-repository-evaluation-facet'});
    }

    me.callParent();
  }
});
