
/*global Ext, NX*/

/**
 * Ssl Certificate detail panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ssl.SslCertificateDetailsPanel', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-sslcertificate-details-panel',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      settingsForm: {
        xtype: 'nx-coreui-sslcertificate-details-form',
        buttons: [
          { text: NX.I18n.get('Ssl_SslCertificateDetailsWindow_Cancel_Button'), action: 'back' }
        ]
      }
    });

    this.callParent();
  }

});
