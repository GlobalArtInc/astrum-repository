
/*global Ext, NX*/

/**
 * Swift repository search contribution.
 *
 *
 */
Ext.define('NX.coreui.controller.SearchSwift', {
    extend: 'NX.app.Controller',
    requires: [
        'NX.I18n'
    ],

    /**
     * @override
     */
    init: function() {
        var me = this,
            search = me.getController('NX.coreui.controller.Search');

        search.registerCriteria([
            {
                id: 'attributes.swift.scope',
                group: NX.I18n.get('SearchSwift_Group'),
                config: {
                    format: 'swift',
                    fieldLabel: NX.I18n.get('SearchSwift_Scope_FieldLabel'),
                    width: 250
                }
            }
        ], me);

        search.registerFilter({
            id: 'swift',
            name: 'Swift',
            text: NX.I18n.get('SearchSwift_Text'),
            description: NX.I18n.get('SearchSwift_Description'),
            readOnly: true,
            criterias: [
                { id: 'format', value: 'swift', hidden: true },
                { id: 'name.raw' },
                { id: 'attributes.swift.scope' },
                { id: 'version' }
            ]
        }, me);
    }

});
