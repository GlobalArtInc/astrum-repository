
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Docker Hosted repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.DockerGroup', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-docker-group',
  requires: [
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.GroupFacet',
    'AS.coreui.view.repository.facet.DockerConnectorFacet',
    'AS.coreui.view.repository.facet.DockerV1Facet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-docker-connector-facet'},
      {xtype: 'as-coreui-repository-docker-v1-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {
        xtype: 'as-coreui-repository-group-facet',
        format: 'docker',
        supportsGroupWrite: true,
        helpHint: AS.I18n.get('GroupDeployment_Docker_Help')
      }
    ];

    me.callParent();
  }
});
