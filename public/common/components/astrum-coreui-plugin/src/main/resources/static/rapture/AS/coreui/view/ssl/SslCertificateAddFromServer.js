
/*global Ext, AS*/

/**
 * Add Ssl Certificate from Server window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslCertificateAddFromServer', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-sslcertificate-add-from-server',
  requires: [
    'AS.Icons',
    'AS.I18n'
  ],

  defaultFocus: 'server',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = {
      xtype: 'as-settingsform',
      settingsFormSubmitOnEnter: true,

      items: [
        {
          xtype: 'textfield',
          fieldLabel: AS.I18n.get('Ssl_SslCertificateAddFromServer_Load_FieldLabel'),
          name: 'server',
          itemId: 'server'
        }
      ],

      buttons: [
        { text: AS.I18n.get('Ssl_SslCertificateList_Load_Button'), action: 'load', formBind: true, bindToEnter: true, ui: 'as-primary' },
        { text: AS.I18n.get('Ssl_SslCertificateAddFromServer_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();
  }
});
