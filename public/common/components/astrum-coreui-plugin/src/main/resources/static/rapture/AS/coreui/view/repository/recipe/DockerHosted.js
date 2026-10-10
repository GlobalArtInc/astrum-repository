
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Docker Hosted repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.DockerHosted', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-docker-hosted',
  requires: [
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.StorageFacetHosted',
    'AS.coreui.view.repository.facet.DockerStorageFacetHosted',
    'AS.coreui.view.repository.facet.DockerConnectorFacet',
    'AS.coreui.view.repository.facet.DockerV1Facet',
    'AS.coreui.view.repository.facet.EvaluationFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-docker-connector-facet'},
      {xtype: 'as-coreui-repository-docker-v1-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-docker-storage-hosted-facet', writePolicy: 'ALLOW'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'}
    ];

    // Only add evaluation facet if feature is enabled
    if (AS.State.getValue('hostedRepositoryEvaluationEnabled')) {
      me.items.splice(me.items.length - 1, 0, {xtype: 'as-coreui-repository-evaluation-facet'});
    }

    me.callParent();
  }
});
