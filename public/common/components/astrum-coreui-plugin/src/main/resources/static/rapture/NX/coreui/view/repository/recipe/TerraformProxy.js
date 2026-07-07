
/*global Ext, NX*/

/**
 * Repository "Settings" form for a terraform Proxy repository
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.repository.recipe.TerraformProxy', {
    extend: 'NX.coreui.view.repository.RepositorySettingsForm',
    alias: 'widget.nx-coreui-repository-terraform-proxy',
    requires: [
        'NX.coreui.view.repository.facet.ProxyFacet',
        'NX.coreui.view.repository.facet.StorageFacet',
        'NX.coreui.view.repository.facet.RoutingRuleFacet',
        'NX.coreui.view.repository.facet.BearerHttpClientFacetWithPreemptiveAuth',
        'NX.coreui.view.repository.facet.NegativeCacheFacet',
        'NX.coreui.view.repository.facet.CleanupPolicyFacet',
        'NX.coreui.view.repository.facet.TerraformProxyFacet'
    ],

    /**
     * @override
     */
    initComponent: function() {
        var me = this;

        me.items = [
            {xtype: 'nx-coreui-repository-terraform-proxy-facet'},
            {xtype: 'nx-coreui-repository-proxy-facet'},
            {xtype: 'nx-coreui-repository-storage-facet'},
            {xtype: 'nx-coreui-repository-routing-rule-facet'},
            {xtype: 'nx-coreui-repository-negativecache-facet'},
            {xtype: 'nx-coreui-repository-cleanup-policy-facet'},
            {xtype: 'nx-coreui-repository-httpclient-facet-with-bearer-token-and-preemptive-auth'}
        ];

        me.callParent();
        me.down('#remoteUrl').setHelpText(NX.I18n.get('Repository_Facet_TerraformFacet_Remote_HelpText'));
    }
});
