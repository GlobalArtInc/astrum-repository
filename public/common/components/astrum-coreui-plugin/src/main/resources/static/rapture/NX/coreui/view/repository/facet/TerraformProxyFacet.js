
/*global Ext, NX*/

/**
 * Configuration for availability of Terraform V1 api.
 */
Ext.define('NX.coreui.view.repository.facet.TerraformProxyFacet', {
    extend: 'Ext.form.FieldContainer',
    alias: 'widget.nx-coreui-repository-terraform-proxy-facet',
    requires: [
        'NX.I18n'
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
