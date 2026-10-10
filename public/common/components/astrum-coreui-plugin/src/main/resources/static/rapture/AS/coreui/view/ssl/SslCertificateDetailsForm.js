
/*global Ext, AS*/

/**
 * Ssl Certificate detail panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslCertificateDetailsForm', {
  extend: 'AS.view.SettingsForm',
  alias: 'widget.as-coreui-sslcertificate-details-form',
  requires: [
    'AS.util.DateFormat',
    'AS.I18n'
  ],

  buttons: undefined,

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.items = [
      {
        xtype: 'panel',
        ui: 'as-subsection',
        title: AS.I18n.get('Ssl_SslCertificateDetailsForm_Subject_Title'),
        items: {
          xtype: 'fieldset',
          margin: 0,
          padding: 0,
          defaults: {
            xtype: 'displayfield',
            labelAlign: 'left',
            renderer: Ext.htmlEncode
          },
          items: [
            { name: 'subjectCommonName', fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_SubjectCommonName_FieldLabel') },
            { name: 'subjectOrganization', fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_SubjectOrganization_FieldLabel') },
            { name: 'subjectOrganizationalUnit', fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_SubjectUnit_FieldLabel') }
          ]
        }
      },
      {
        xtype: 'panel',
        ui: 'as-subsection',
        title: AS.I18n.get('Ssl_SslCertificateDetailsForm_Issuer_Title'),
        items: {
          xtype: 'fieldset',
          margin: 0,
          padding: 0,
          defaults: {
            xtype: 'displayfield',
            labelAlign: 'left',
            renderer: Ext.htmlEncode
          },
          items: [
            { name: 'issuerCommonName', fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_IssuerName_FieldLabel') },
            { name: 'issuerOrganization', fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_IssuerOrganization_FieldLabel') },
            { name: 'issuerOrganizationalUnit', fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_IssuerUnit_FieldLabel') }
          ]
        }
      },
      {
        xtype: 'panel',
        ui: 'as-subsection',
        title: AS.I18n.get('Ssl_SslCertificateDetailsForm_Certificate_Title'),
        items: [
          {
            xtype: 'fieldset',
            margin: 0,
            padding: '0 0 10px 0',
            defaults: {
              xtype: 'displayfield',
              labelAlign: 'left'
            },
            items: [
              {
                name: 'issuedOn',
                fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_CertificateIssuedOn_FieldLabel'),
                renderer: AS.util.DateFormat.timestampRenderer()
              },
              {
                name: 'expiresOn',
                fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_CertificateValidUntil_FieldLabel'),
                renderer: AS.util.DateFormat.timestampRenderer()
              },
              {
                name: 'fingerprint',
                fieldLabel: AS.I18n.get('Ssl_SslCertificateDetailsForm_CertificateFingerprint_FieldLabel')
              }
            ]
          },
          {
            xtype: 'label',
            html: AS.I18n.get('Ssl_SslCertificateDetailsForm_RetrievedUntrustedConnection_Html')
          }
        ]
      }
    ];

    me.callParent();
  }
});
