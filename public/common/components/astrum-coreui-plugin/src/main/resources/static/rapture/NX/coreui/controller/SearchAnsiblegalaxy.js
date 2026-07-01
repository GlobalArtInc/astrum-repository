

/**
 * Ansible Galaxy repository search contribution.
 */
Ext.define('NX.coreui.controller.SearchAnsiblegalaxy', {
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
                id: 'assets.attributes.ansiblegalaxy.namespace',
                group: NX.I18n.get('SearchAnsiblegalaxy_Group'),
                config: {
                    format: 'ansiblegalaxy',
                    fieldLabel: NX.I18n.get('SearchAnsiblegalaxy_Namespace_FieldLabel'),
                    width: 250
                }
            },
            {
                id: 'assets.attributes.ansiblegalaxy.name',
                group: NX.I18n.get('SearchAnsiblegalaxy_Group'),
                config: {
                    format: 'ansiblegalaxy',
                    fieldLabel: NX.I18n.get('SearchAnsiblegalaxy_Name_FieldLabel'),
                    width: 250
                }
            },
            {
                id: 'assets.attributes.ansiblegalaxy.version',
                group: NX.I18n.get('SearchAnsiblegalaxy_Group'),
                config: {
                    format: 'ansiblegalaxy',
                    fieldLabel: NX.I18n.get('SearchAnsiblegalaxy_Version_FieldLabel'),
                    width: 250
                }
            }
        ], me);

        search.registerFilter({
            id: 'ansiblegalaxy',
            name: 'Ansible Galaxy',
            text: NX.I18n.get('SearchAnsiblegalaxy_Text'),
            description: NX.I18n.get('SearchAnsiblegalaxy_Description'),
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
