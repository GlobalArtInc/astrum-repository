
/**
 * Configuration specific to content signing of YUM repositories.
 *
 * @since 3.30
 */
Ext.define('AS.coreui.view.repository.facet.YumSigningFacet', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-repository-yum-signing-facet',
  requires: [
    'AS.I18n',
    'AS.ext.form.field.Password'
  ],

  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'fieldset',
        cls: 'as-form-section',
        title: AS.I18n.get('Repository_Facet_YumSigningFacet_Title'),
        items: [
          {
            xtype: 'label',
            html: AS.I18n.get('Repository_Facet_YumSigningFacet_Hint'),
            margin: '0 0 0 -10'
          },
          {
            xtype: 'textareafield',
            name: 'attributes.yumSigning.keypair',
            fieldLabel: AS.I18n.get('Repository_Facet_YumSigningFacet_GPG_Keypair_FieldLabel'),
            helpText: AS.I18n.get('Repository_Facet_YumSigningFacet_GPG_Keypair_HelpText')
          },
          {
            xtype: 'as-password',
            name: 'attributes.yumSigning.passphrase',
            fieldLabel: AS.I18n.get('Repository_Facet_YumSigningFacet_GPG_Passphrase_FieldLabel'),
            allowBlank: true
          }
        ]
      }
    ];

    me.callParent();
  }

});
