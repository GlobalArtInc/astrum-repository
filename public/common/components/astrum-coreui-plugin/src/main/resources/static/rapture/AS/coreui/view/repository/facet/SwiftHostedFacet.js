

/**
 * Configuration specific to swift repositories.
 */
Ext.define('AS.coreui.view.repository.facet.SwiftHostedFacet', {
    extend: 'Ext.form.FieldContainer',
    alias: 'widget.as-swiftui-repository-swifthosted-facet',
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
                title: AS.I18n.get('Repository_Facet_SwiftFacet_Title'),
                items: []
            }
        ];

        me.callParent();
    }

});
