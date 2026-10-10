
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Pub Hosted repository.
 */
Ext.define('AS.coreui.view.repository.recipe.PubHosted', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-pub-hosted',
  requires: [
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.StorageFacetHosted',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-storage-hosted-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'}
    ];

    me.callParent();
  }
});
