
/*global Ext, NX*/

/**
 * Repository "Settings" form for a Docker Hosted repository.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.DockerGroup', {
  extend: 'NX.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.nx-coreui-repository-docker-group',
  requires: [
    'NX.coreui.view.repository.facet.StorageFacet',
    'NX.coreui.view.repository.facet.GroupFacet',
    'NX.coreui.view.repository.facet.DockerConnectorFacet',
    'NX.coreui.view.repository.facet.DockerV1Facet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {xtype: 'nx-coreui-repository-docker-connector-facet'},
      {xtype: 'nx-coreui-repository-docker-v1-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {
        xtype: 'nx-coreui-repository-group-facet',
        format: 'docker',
        supportsGroupWrite: true,
        helpHint: NX.I18n.get('GroupDeployment_Docker_Help')
      }
    ];

    me.callParent();
  }
});
