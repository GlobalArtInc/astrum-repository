
/*global Ext, NX*/

/**
 * Add Ssl Certificate from Server window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ssl.SslCertificateAddFromServer', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-sslcertificate-add-from-server',
  requires: [
    'NX.Icons',
    'NX.I18n'
  ],

  defaultFocus: 'server',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'nx-settingsform',
      settingsFormSubmitOnEnter: true,

      items: [
        {
          xtype: 'textfield',
          fieldLabel: NX.I18n.get('Ssl_SslCertificateAddFromServer_Load_FieldLabel'),
          name: 'server',
          itemId: 'server'
        }
      ],

      buttons: [
        { text: NX.I18n.get('Ssl_SslCertificateList_Load_Button'), action: 'load', formBind: true, bindToEnter: true, ui: 'nx-primary' },
        { text: NX.I18n.get('Ssl_SslCertificateAddFromServer_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();
  }
});
