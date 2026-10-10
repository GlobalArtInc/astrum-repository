
/**
 * Configuration specific to terraform signing.
 */
Ext.define('AS.coreui.view.repository.facet.TerraformSigningFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-terraformsigning-facet',
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
        items: [
          {
            xtype:'textareafield',
            name: 'attributes.terraformSigning.keypair',
            fieldLabel: AS.I18n.get('Repository_Facet_TerraformSigningFacet_Keypair_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_TerraformSigningFacet_Keypair_HelpText'),
            allowBlank: false,
            grow: true
          },
          {
            xtype:'as-password',
            name: 'attributes.terraformSigning.passphrase',
            fieldLabel: AS.I18n.get('Repository_Facet_TerraformSigningFacet_Passphrase_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    ];

    me.callParent();
  }

});
