
/*global Ext, AS*/

/**
 * Repository "Settings" form for a Maven Hosted repository.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.RawHosted', {
  extend: 'AS.coreui.view.repository.RepositorySettingsForm',
  alias: 'widget.as-coreui-repository-raw-hosted',
  requires: [
    'AS.coreui.view.repository.facet.RawFacet',
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
      {xtype: 'as-coreui-repository-raw-facet'},
      {xtype: 'as-coreui-repository-storage-facet', strictContentTypeValidation: false},
      {xtype: 'as-coreui-repository-storage-hosted-facet', writePolicy: 'ALLOW'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'}
    ];

    // Only add evaluation facet if feature is enabled
    if (AS.State.getValue('hostedRepositoryEvaluationEnabled')) {
      me.items.splice(me.items.length - 1, 0, {xtype: 'as-coreui-repository-evaluation-facet'});
    }

    me.callParent();
  }
});
