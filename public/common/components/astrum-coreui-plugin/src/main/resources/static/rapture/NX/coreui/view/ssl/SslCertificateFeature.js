
/*global Ext, NX*/

/**
 * Ssl Certificate Set feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ssl.SslCertificateFeature', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-sslcertificate-feature',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function() {
    Ext.apply(this, {
      iconName: 'sslcertificate-default',

      masters: [
        { xtype: 'nx-coreui-sslcertificate-list' }
      ],

      tabs: { xtype: 'nx-coreui-sslcertificate-details' },

      nxActions: [
        {
          xtype: 'button',
          text: NX.I18n.get('Ssl_SslCertificateFeature_Delete_Button'),
          iconCls: 'x-fa fa-trash',
          action: 'delete',
          disabled: true
        }
      ]
    });

    this.callParent();
  }
});
