
/*global Ext, NX*/

/**
 * Terraform repository search contribution.
 *
 *
 */
Ext.define('NX.coreui.controller.SearchTerraform', {
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
                id: 'assets.attributes.terraform.provider',
                group: NX.I18n.get('SearchTerraform_Group'),
                config: {
                    format: 'terraform',
                    fieldLabel: NX.I18n.get('SearchTerraform_Provider_FieldLabel'),
                    width: 250
                }
            },
            {
                id: 'assets.attributes.terraform.namespace',
                group: NX.I18n.get('SearchTerraform_Group'),
                config: {
                    format: 'terraform',
                    fieldLabel: NX.I18n.get('SearchTerraform_Namespace_FieldLabel'),
                    width: 250
                }
            },
            {
                id: 'assets.attributes.terraform.name',
                group: NX.I18n.get('SearchTerraform_Group'),
                config: {
                    format: 'terraform',
                    fieldLabel: NX.I18n.get('SearchTerraform_Name_FieldLabel'),
                    width: 250
                }
            }
        ], me);

        search.registerFilter({
            id: 'terraform',
            name: 'Terraform',
            text: NX.I18n.get('SearchTerraform_Text'),
            description: NX.I18n.get('SearchTerraform_Description'),
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
