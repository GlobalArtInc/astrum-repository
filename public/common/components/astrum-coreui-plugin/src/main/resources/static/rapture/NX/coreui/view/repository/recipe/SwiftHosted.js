
/*global Ext, NX*/

/**
 * Repository "Settings" form for a swift Hosted repository.
 */
Ext.define('NX.coreui.view.repository.recipe.SwiftHosted', {
    extend: 'NX.coreui.view.repository.RepositorySettingsForm',
    alias: 'widget.nx-coreui-repository-swift-hosted',
    requires: [
        'NX.coreui.view.repository.facet.StorageFacet',
        'NX.coreui.view.repository.facet.StorageFacetHostedSwift',
        'NX.coreui.view.repository.facet.EvaluationFacet',
    'NX.coreui.view.repository.facet.CleanupPolicyFacet'
    ],

    /**
     * @override
     */
    initComponent: function() {
        var me = this;

        me.items = [
            {xtype: 'nx-coreui-repository-storage-facet'},
      {xtype: 'nx-coreui-repository-storage-hosted-swift-facet'},
      {xtype: 'nx-coreui-repository-cleanup-policy-facet'}
        ];

        // Only add evaluation facet if feature is enabled
    if (NX.State.getValue('hostedRepositoryEvaluationEnabled')) {
      me.items.splice(me.items.length - 1, 0, {xtype: 'nx-coreui-repository-evaluation-facet'});
    }

        me.callParent();
    }
});
