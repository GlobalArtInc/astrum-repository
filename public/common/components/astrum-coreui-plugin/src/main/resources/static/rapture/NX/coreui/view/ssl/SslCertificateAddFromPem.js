
/*global Ext, NX*/

/**
 * Add Ssl Certificate from PEM window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ssl.SslCertificateAddFromPem', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-sslcertificate-add-from-pem',
  requires: [
    'NX.Icons',
    'NX.I18n'
  ],

  defaultFocus: 'pem',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'nx-settingsform',

      items: [
        {
          xtype: 'textareafield',
          name: 'pem',
          itemId: 'pem',
          cls: 'nx-monospace-field',
          height: '16rem'
        }
      ],

      buttons: [
        { text: NX.I18n.get('Ssl_SslCertificateList_New_Button'), action: 'load', formBind: true, ui: 'nx-primary' },
        { text: NX.I18n.get('Ssl_SslCertificateAddFromPem_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();
  }

});
