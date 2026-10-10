

/**
 * Configuration specific to terraform repositories.
 */
Ext.define('AS.coreui.view.repository.facet.TerraformHostedFacet', {
    extend: 'Ext.form.FieldContainer',
    alias: 'widget.as-terraformui-repository-terraformhosted-facet',
    requires: [
        'AS.I18n'
    ],
    /**
     * @override
     */
    initComponent: function() {
        var me = this;

        me.items = [
            {
                xtype: 'fieldset',
                cls: 'as-form-section',
                title: AS.I18n.get('Repository_Facet_TerraformFacet_Title'),
                items: []
            }
        ];

        me.callParent();
    }

});
