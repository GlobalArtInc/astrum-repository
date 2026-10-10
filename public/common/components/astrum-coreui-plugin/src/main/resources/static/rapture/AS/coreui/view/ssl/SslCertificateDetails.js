
/*global Ext, AS*/

/**
 * Ssl Certificate detail panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ssl.SslCertificateDetails', {
  extend: 'AS.view.SettingsPanel',
  alias: 'widget.as-coreui-sslcertificate-details',

  title: 'Summary',
  settingsForm: { xtype: 'as-coreui-sslcertificate-details-form' },

  dockedItems: null
});
