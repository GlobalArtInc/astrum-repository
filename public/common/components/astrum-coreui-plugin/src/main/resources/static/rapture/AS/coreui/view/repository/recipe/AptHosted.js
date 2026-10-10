
/**
 * Repository settings form for an apt repository
 *
 * @since 3.17
 */
Ext.define('AS.coreui.view.repository.recipe.AptHosted', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-apt-hosted',
  requires: [
    'AS.coreui.view.repository.facet.AptHostedFacet',
    'AS.coreui.view.repository.facet.AptSigningFacet',
    'AS.coreui.view.repository.facet.StorageFacet',
    'AS.coreui.view.repository.facet.StorageFacetHosted',
    'AS.coreui.view.repository.facet.EvaluationFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {xtype: 'as-aptui-repository-apthosted-facet'},
      {xtype: 'as-aptui-repository-aptsigning-facet', allowBlank: false},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-storage-hosted-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'}
    ];

    // Only add evaluation facet if feature is enabled
    if (AS.State.getValue('hostedRepositoryEvaluationEnabled')) {
      me.items.splice(me.items.length - 1, 0, {xtype: 'as-coreui-repository-evaluation-facet'});
    }

    me.callParent();
  }
});
