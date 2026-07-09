
/*global Ext, NX*/

/**
 * Ssl Certificate detail panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ssl.SslCertificateDetails', {
  extend: 'NX.view.SettingsPanel',
  alias: 'widget.nx-coreui-sslcertificate-details',

  title: 'Summary',
  settingsForm: { xtype: 'nx-coreui-sslcertificate-details-form' },

  dockedItems: null
});
