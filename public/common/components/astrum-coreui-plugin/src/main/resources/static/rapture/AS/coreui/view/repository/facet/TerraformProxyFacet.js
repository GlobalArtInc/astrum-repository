
/*global Ext, AS*/

/**
 * Configuration for availability of Terraform V1 api.
 */
Ext.define('AS.coreui.view.repository.facet.TerraformProxyFacet', {
    extend: 'Ext.form.FieldContainer',
    alias: 'widget.as-coreui-repository-terraform-proxy-facet',
    requires: [
        'AS.I18n'
    ],

    /**
     * @override
     */
    initComponent: function() {
        const me = this;

        me.items = [];

        me.callParent();
    }

});
