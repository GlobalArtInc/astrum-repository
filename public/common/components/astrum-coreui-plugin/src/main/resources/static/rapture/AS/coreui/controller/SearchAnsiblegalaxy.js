

/**
 * Ansible Galaxy repository search contribution.
 */
Ext.define('AS.coreui.controller.SearchAnsiblegalaxy', {
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
                id: 'assets.attributes.ansiblegalaxy.namespace',
                group: AS.I18n.get('SearchAnsiblegalaxy_Group'),
                config: {
                    format: 'ansiblegalaxy',
                    fieldLabel: AS.I18n.get('SearchAnsiblegalaxy_Namespace_FieldLabel'),
                    width: 250
                }
            },
            {
                id: 'assets.attributes.ansiblegalaxy.name',
                group: AS.I18n.get('SearchAnsiblegalaxy_Group'),
                config: {
                    format: 'ansiblegalaxy',
                    fieldLabel: AS.I18n.get('SearchAnsiblegalaxy_Name_FieldLabel'),
                    width: 250
                }
            },
            {
                id: 'assets.attributes.ansiblegalaxy.version',
                group: AS.I18n.get('SearchAnsiblegalaxy_Group'),
                config: {
                    format: 'ansiblegalaxy',
                    fieldLabel: AS.I18n.get('SearchAnsiblegalaxy_Version_FieldLabel'),
                    width: 250
                }
            }
        ], me);

        search.registerFilter({
            id: 'ansiblegalaxy',
            name: 'Ansible Galaxy',
            text: AS.I18n.get('SearchAnsiblegalaxy_Text'),
            description: AS.I18n.get('SearchAnsiblegalaxy_Description'),
            readOnly: true,
            criterias: [
                { id: 'format', value: 'ansiblegalaxy', hidden: true },
                { id: 'assets.attributes.ansiblegalaxy.namespace' },
                { id: 'assets.attributes.ansiblegalaxy.name' },
                { id: 'assets.attributes.ansiblegalaxy.version' }
            ]
        }, me);
    }

});
