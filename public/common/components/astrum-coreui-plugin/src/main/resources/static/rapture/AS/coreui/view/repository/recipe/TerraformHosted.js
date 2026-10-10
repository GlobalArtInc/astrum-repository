
/*global Ext, AS*/

/**
 * Repository "Settings" form for a terraform Hosted repository.
 */
Ext.define('AS.coreui.view.repository.recipe.TerraformHosted', {
    extend: 'AS.coreui.view.repository.RepositorySettingsForm',
    alias: 'widget.as-coreui-repository-terraform-hosted',
    requires: [
        'AS.coreui.view.repository.facet.TerraformSigningFacet',
        'AS.coreui.view.repository.facet.StorageFacet',
        'AS.coreui.view.repository.facet.StorageFacetHostedTerraform',
        'AS.coreui.view.repository.facet.EvaluationFacet',
    'AS.coreui.view.repository.facet.CleanupPolicyFacet'
    ],

    /**
     * @override
     */
    initComponent: function() {
        var me = this;

        me.items = [
            {xtype: 'as-coreui-repository-terraformsigning-facet'},
      {xtype: 'as-coreui-repository-storage-facet'},
      {xtype: 'as-coreui-repository-storage-hosted-terraform-facet'},
      {xtype: 'as-coreui-repository-cleanup-policy-facet'}
        ];

        // Only add evaluation facet if feature is enabled
    if (AS.State.getValue('hostedRepositoryEvaluationEnabled')) {
      me.items.splice(me.items.length - 1, 0, {xtype: 'as-coreui-repository-evaluation-facet'});
    }

        me.callParent();
    }
});
