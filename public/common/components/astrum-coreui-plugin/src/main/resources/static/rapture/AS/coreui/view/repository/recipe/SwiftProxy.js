
/**
 * Repository "Settings" form for a Swift Proxy repository
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.repository.recipe.SwiftProxy', {
    extend: 'AS.coreui.view.repository.RepositorySettingsForm',
    alias: 'widget.as-coreui-repository-swift-proxy',
    requires: [
        'AS.coreui.view.repository.facet.ProxyFacet',
        'AS.coreui.view.repository.facet.StorageFacet',
        'AS.coreui.view.repository.facet.RoutingRuleFacet',
        'AS.coreui.view.repository.facet.BearerHttpClientFacet',
        'AS.coreui.view.repository.facet.NegativeCacheFacet',
        'AS.coreui.view.repository.facet.CleanupPolicyFacet',
        'AS.coreui.view.repository.facet.SwiftProxyFacet'
    ],

    /**
     * @override
     */
    initComponent: function() {
        var me = this;

        me.items = [
            {xtype: 'as-coreui-repository-swift-proxy-facet'},
            {xtype: 'as-coreui-repository-proxy-facet'},
            {xtype: 'as-coreui-repository-storage-facet'},
            {xtype: 'as-coreui-repository-routing-rule-facet'},
            {xtype: 'as-coreui-repository-negativecache-facet'},
            {xtype: 'as-coreui-repository-cleanup-policy-facet'},
            {xtype: 'as-coreui-repository-httpclient-facet-with-bearer-token'}
        ];

        me.callParent();
        me.down('#remoteUrl').setHelpText(AS.I18n.get('Repository_Facet_SwiftFacet_Remote_HelpText'));
    }
});