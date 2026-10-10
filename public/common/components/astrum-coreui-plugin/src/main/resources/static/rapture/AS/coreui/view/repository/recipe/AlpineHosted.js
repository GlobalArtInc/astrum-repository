
/**
 * Repository "Settings" form for an Alpine Hosted repository
 */
Ext.define('AS.coreui.view.repository.recipe.AlpineHosted', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-alpine-hosted',
  requires: [
    'AS.coreui.view.repository.facet.AlpineSigningFacet',
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
      {xtype: 'as-coreui-repository-alpinesigning-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-storage-hosted-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'}
    ];

    me.callParent();
  }
});
