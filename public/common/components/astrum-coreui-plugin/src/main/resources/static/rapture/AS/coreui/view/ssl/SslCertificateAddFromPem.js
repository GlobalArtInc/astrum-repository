
/*global Ext, AS*/

/**
 * Add Ssl Certificate from PEM window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslCertificateAddFromPem', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-sslcertificate-add-from-pem',
  requires: [
    'AS.Icons',
    'AS.I18n'
  ],

  defaultFocus: 'pem',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'as-settingsform',

      items: [
        {
          xtype: 'textareafield',
          name: 'pem',
          itemId: 'pem',
          cls: 'as-monospace-field',
          height: '16rem'
        }
      ],

      buttons: [
        { text: AS.I18n.get('Ssl_SslCertificateList_New_Button'), action: 'load', formBind: true, ui: 'as-primary' },
        { text: AS.I18n.get('Ssl_SslCertificateAddFromPem_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();
  }

});
