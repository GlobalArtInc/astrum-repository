
/*global Ext, NX*/

/**
 * Repository "Settings" form for a terraform Hosted repository.
 */
Ext.define('NX.coreui.view.repository.recipe.TerraformHosted', {
    extend: 'NX.coreui.view.repository.RepositorySettingsForm',
    alias: 'widget.nx-coreui-repository-terraform-hosted',
    requires: [
        'NX.coreui.view.repository.facet.TerraformSigningFacet',
        'NX.coreui.view.repository.facet.StorageFacet',
        'NX.coreui.view.repository.facet.StorageFacetHostedTerraform',
        'NX.coreui.view.repository.facet.EvaluationFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet'
    ],

    /**
     * @override
     */
    initComponent: function() {
        var me = this;

        me.items = [
            {xtype: 'nx-coreui-repository-terraformsigning-facet'},
      {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-storage-hosted-terraform-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'}
        ];

        // Only add evaluation facet if feature is enabled
    if (NX.State.getValue('hostedRepositoryEvaluationEnabled')) {
      me.items.splice(me.items.length - 1, 0, {xtype: 'nx-coreui-repository-evaluation-facet'});
    }

        me.callParent();
    }
});
