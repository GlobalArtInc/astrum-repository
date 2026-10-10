
/*global Ext, AS*/

/**
 * Ssl Certificate detail window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslCertificateDetailsWindow', {
  extend: 'AS.view.AddWindow',
  alias: 'widget.as-coreui-sslcertificate-details-window',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      ui: 'as-inset',

      title: AS.I18n.get('Ssl_SslCertificateDetailsWindow_Title'),

      items: {
        xtype: 'as-coreui-sslcertificate-details-form',
        frame: false,
        buttons: [
          { text: AS.I18n.get('Ssl_SslCertificateDetailsWindow_Cancel_Button'),
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
