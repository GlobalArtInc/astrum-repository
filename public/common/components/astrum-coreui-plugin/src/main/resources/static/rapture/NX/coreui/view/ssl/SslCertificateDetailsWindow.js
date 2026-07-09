
/*global Ext, NX*/

/**
 * Ssl Certificate detail window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ssl.SslCertificateDetailsWindow', {
  extend: 'NX.view.AddWindow',
  alias: 'widget.nx-coreui-sslcertificate-details-window',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      ui: 'nx-inset',

      title: NX.I18n.get('Ssl_SslCertificateDetailsWindow_Title'),

      items: {
        xtype: 'nx-coreui-sslcertificate-details-form',
        frame: false,
        buttons: [
          { text: NX.I18n.get('Ssl_SslCertificateDetailsWindow_Cancel_Button'),
            handler: function () {
              this.up('window').close();
            }
          }
        ]
      }
    });

    this.callParent();
  }

});
