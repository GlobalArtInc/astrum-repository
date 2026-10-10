
/*global Ext, AS*/

/**
 * Terraform repository search contribution.
 *
 *
 */
Ext.define('AS.coreui.controller.SearchTerraform', {
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
                id: 'assets.attributes.terraform.provider',
                group: AS.I18n.get('SearchTerraform_Group'),
                config: {
                    format: 'terraform',
                    fieldLabel: AS.I18n.get('SearchTerraform_Provider_FieldLabel'),
                    width: 250
                }
            },
            {
                id: 'assets.attributes.terraform.namespace',
                group: AS.I18n.get('SearchTerraform_Group'),
                config: {
                    format: 'terraform',
                    fieldLabel: AS.I18n.get('SearchTerraform_Namespace_FieldLabel'),
                    width: 250
                }
            },
            {
                id: 'assets.attributes.terraform.name',
                group: AS.I18n.get('SearchTerraform_Group'),
                config: {
                    format: 'terraform',
                    fieldLabel: AS.I18n.get('SearchTerraform_Name_FieldLabel'),
                    width: 250
                }
            }
        ], me);

        search.registerFilter({
            id: 'terraform',
            name: 'Terraform',
            text: AS.I18n.get('SearchTerraform_Text'),
            description: AS.I18n.get('SearchTerraform_Description'),
            readOnly: true,
            criterias: [
                { id: 'format', value: 'terraform', hidden: true },
                { id: 'assets.attributes.terraform.provider' },
                { id: 'assets.attributes.terraform.namespace' },
                { id: 'assets.attributes.terraform.name' },
                { id: 'version' }
            ]
        }, me);
    }

});
