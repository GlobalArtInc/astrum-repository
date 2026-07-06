
/**
 * Configuration specific to content signing of YUM repositories.
 *
 * @since 3.30
 */
Ext.define('NX.coreui.view.repository.facet.YumSigningFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-repository-yum-signing-facet',
  requires: [
    'NX.I18n',
    'NX.ext.form.field.Password'
  ],

  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'nx-form-section',
        title: NX.I18n.get('Repository_Facet_YumSigningFacet_Title'),
        items: [
          {
            xtype: 'label',
            html: NX.I18n.get('Repository_Facet_YumSigningFacet_Hint'),
            margin: '0 0 0 -10'
          },
          {
            xtype: 'textareafield',
            name: 'attributes.yumSigning.keypair',
            fieldLabel: NX.I18n.get('Repository_Facet_YumSigningFacet_GPG_Keypair_FieldLabel'),
            helpText: NX.I18n.get('Repository_Facet_YumSigningFacet_GPG_Keypair_HelpText')
          },
          {
            xtype: 'nx-password',
            name: 'attributes.yumSigning.passphrase',
            fieldLabel: NX.I18n.get('Repository_Facet_YumSigningFacet_GPG_Passphrase_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    ];

    me.callParent();
  }

});
