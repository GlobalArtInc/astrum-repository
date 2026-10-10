
/*global Ext, AS*/

/**
 * Ssl Certificate detail panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslCertificateDetailsPanel', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-sslcertificate-details-panel',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      settingsForm: {
        xtype: 'as-coreui-sslcertificate-details-form',
        buttons: [
          { text: AS.I18n.get('Ssl_SslCertificateDetailsWindow_Cancel_Button'), action: 'back' }
        ]
      }
    });

    this.callParent();
  }

});
