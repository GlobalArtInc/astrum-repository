

/**
 * Configuration specific to terraform repositories.
 */
Ext.define('NX.coreui.view.repository.facet.TerraformHostedFacet', {
    extend: 'Ext.form.FieldContainer',
    alias: 'widget.nx-terraformui-repository-terraformhosted-facet',
    requires: [
        'NX.I18n'
    ],
    /**
     * @override
     */
    initComponent: function() {
        var me = this;

        me.items = [
            {
                xtype: 'fieldset',
                cls: 'nx-form-section',
                title: NX.I18n.get('Repository_Facet_TerraformFacet_Title'),
                items: []
            }
        ];

        me.callParent();
    }

});
