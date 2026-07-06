
/**
 * Configuration specific to terraform signing.
 */
Ext.define('NX.coreui.view.repository.facet.TerraformSigningFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-terraformsigning-facet',
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
        title: NX.I18n.get('Repository_Facet_TerraformFacet_Title'),
        items: [
          {
            xtype:'textareafield',
            name: 'attributes.terraformSigning.keypair',
            fieldLabel: NX.I18n.get('Repository_Facet_TerraformSigningFacet_Keypair_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_TerraformSigningFacet_Keypair_HelpText'),
            allowBlank: false,
            grow: true
          },
          {
            xtype:'nx-password',
            name: 'attributes.terraformSigning.passphrase',
            fieldLabel: NX.I18n.get('Repository_Facet_TerraformSigningFacet_Passphrase_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    ];

    me.callParent();
  }

});
