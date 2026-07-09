
/*global Ext, NX*/

/**
 * Ssl Certificate detail panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ssl.SslCertificateDetailsForm', {
  extend: 'NX.view.SettingsForm',
  alias: 'widget.nx-coreui-sslcertificate-details-form',
  requires: [
    'NX.util.DateFormat',
    'NX.I18n'
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
        ui: 'nx-subsection',
        title: NX.I18n.get('Ssl_SslCertificateDetailsForm_Subject_Title'),
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
            { name: 'subjectCommonName', fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_SubjectCommonName_FieldLabel') },
            { name: 'subjectOrganization', fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_SubjectOrganization_FieldLabel') },
            { name: 'subjectOrganizationalUnit', fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_SubjectUnit_FieldLabel') }
          ]
        }
      },
      {
        xtype: 'panel',
        ui: 'nx-subsection',
        title: NX.I18n.get('Ssl_SslCertificateDetailsForm_Issuer_Title'),
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
            { name: 'issuerCommonName', fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_IssuerName_FieldLabel') },
            { name: 'issuerOrganization', fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_IssuerOrganization_FieldLabel') },
            { name: 'issuerOrganizationalUnit', fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_IssuerUnit_FieldLabel') }
          ]
        }
      },
      {
        xtype: 'panel',
        ui: 'nx-subsection',
        title: NX.I18n.get('Ssl_SslCertificateDetailsForm_Certificate_Title'),
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
                fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_CertificateIssuedOn_FieldLabel'),
                renderer: NX.util.DateFormat.timestampRenderer()
              },
              {
                name: 'expiresOn',
                fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_CertificateValidUntil_FieldLabel'),
                renderer: NX.util.DateFormat.timestampRenderer()
              },
              {
                name: 'fingerprint',
                fieldLabel: NX.I18n.get('Ssl_SslCertificateDetailsForm_CertificateFingerprint_FieldLabel')
              }
            ]
          },
          {
            xtype: 'label',
            html: NX.I18n.get('Ssl_SslCertificateDetailsForm_RetrievedUntrustedConnection_Html')
          }
        ]
      }
    ];

    me.callParent();
  }
});
