
/*global Ext, AS*/

/**
 * Swift repository search contribution.
 *
 *
 */
Ext.define('AS.coreui.controller.SearchSwift', {
    extend: 'AS.app.Controller',
    requires: [
        'AS.I18n'
    ],

    /**
     * @override
     */
    init: function() {
        var me = this,
            search = me.getController('AS.coreui.controller.Search');

        search.registerCriteria([
            {
                id: 'attributes.swift.scope',
                group: AS.I18n.get('SearchSwift_Group'),
                config: {
                    format: 'swift',
                    fieldLabel: AS.I18n.get('SearchSwift_Scope_FieldLabel'),
                    width: 250
                }
            }
        ], me);

        search.registerFilter({
            id: 'swift',
            name: 'Swift',
            text: AS.I18n.get('SearchSwift_Text'),
            description: AS.I18n.get('SearchSwift_Description'),
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
