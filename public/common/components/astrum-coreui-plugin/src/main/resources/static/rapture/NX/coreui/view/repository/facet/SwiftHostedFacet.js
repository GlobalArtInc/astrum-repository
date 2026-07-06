

/**
 * Configuration specific to swift repositories.
 */
Ext.define('NX.coreui.view.repository.facet.SwiftHostedFacet', {
    extend: 'Ext.form.FieldContainer',
    alias: 'widget.nx-swiftui-repository-swifthosted-facet',
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
                title: NX.I18n.get('Repository_Facet_SwiftFacet_Title'),
                items: []
            }
        ];

        me.callParent();
    }

});
