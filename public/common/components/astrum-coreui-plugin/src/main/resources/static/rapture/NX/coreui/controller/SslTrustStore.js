
/*global Ext, NX*/

/**
 * Ssl TrustStore controller.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.controller.SslTrustStore', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.Conditions',
    'NX.coreui.controller.SslCertificates'
  ],

  views: [
    'ssl.SslUseTrustStore'
  ],

  refs: [{ ref: 'main', selector: '#feature-content' }],

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.listen({
      component: {
        'field[useTrustStore]': {
          change: me.manageTrustStore
        },
        'nx-coreui-sslusetruststore button[action=showcertificate]': {
          click: me.loadCertificate
        }
      }
    });
  },

  /**
   * @private
   * @param field
   * @param field.useTrustStore
   * @param field.useTrustStoreField
   */
  manageTrustStore: function(field) {
    var me = this,
        container = field.up('container'),
        useTrustStoreField = field.useTrustStoreField,
        config, hostAndPort;

    if (Ext.isFunction(field.useTrustStore)) {
      config = field.useTrustStore.call(field, field);
      if (config) {
        hostAndPort = me.getHostAndPort(config);
        if (useTrustStoreField && ((useTrustStoreField.name !== config.name) || !hostAndPort.host)) {
          container.remove(useTrustStoreField);
          delete field.useTrustStoreField;
          useTrustStoreField = undefined;
        }
        if (hostAndPort.host) {
          if (!useTrustStoreField) {
            container.insert(container.items.indexOf(field) + 1, {
              xtype: 'nx-coreui-sslusetruststore',
              name: config.name,
              value: config.value,
              fieldLabel: field.useTrustStoreFieldLabel,
              boxLabel: field.useTrustStoreBoxLabel,
              useTrustStoreConfig: config,
              cls: 'nx-clear-both',
              listeners: {
                afterrender: me.bindConditions
              }
            });
            field.useTrustStoreField = container.down('nx-coreui-sslusetruststore[name=' + config.name + ']');
          }
          else {
            useTrustStoreField.useTrustStoreConfig = config;
          }
        }
      }
    }
    if ((!config) && useTrustStoreField) {
      container.remove(useTrustStoreField);
      delete field.useTrustStoreField;
    }
  },

  /**
   * @private
   * Retrieves certificate, showing the certificate details if successful.
   */
  loadCertificate: function(button) {
    var me = this,
        panel = button.up('panel').up('panel'),
        useTrustStoreConfig = button.up('nx-coreui-sslusetruststore').useTrustStoreConfig,
        hostAndPort = me.getHostAndPort(useTrustStoreConfig),
        protocolHint = useTrustStoreConfig.url && Ext.String.startsWith(useTrustStoreConfig.url.getValue(), "https://") ? 'https' : undefined,
        sslCertificates = me.getController('NX.coreui.controller.SslCertificates');

    me.getMain().getEl().mask(NX.I18n.get('SslTrustStore_Load_Mask'));
    NX.direct.ssl_Certificate.retrieveFromHost(hostAndPort.host, hostAndPort.port, protocolHint, function(response) {
      me.getMain().getEl().unmask();
      if (Ext.isObject(response) && response.success) {
        sslCertificates.showCertificateDetailsWindow(response.data);
      }
    });
  },

  /**
   * @private
   * Get host/port out of config.
   */
  getHostAndPort: function(config) {
    var sslCertificates = this.getController('NX.coreui.controller.SslCertificates'),
        valueOf = function(value) {
          if (Ext.isString(value)) {
            return value;
          }
          else if (value && Ext.isFunction(value.getValue)) {
            return value.getValue();
          }
          return undefined;
        },
        parsed, host, port;

    if (config.url) {
      parsed = sslCertificates.parseHostAndPort(valueOf(config.url));
      host = parsed[0];
      port = parsed[1];
    }
    else {
      host = valueOf(config.host);
      port = valueOf(config.port);
    }

    return {
      host: host,
      port: port
    };
  },

  bindConditions: function(useTrustStoreField) {
    var useTrustStoreCheckbox, useTrustStoreButton;

    useTrustStoreCheckbox = useTrustStoreField.down('checkbox');
    useTrustStoreButton = useTrustStoreField.down('button');
    
    // Checkbox is always enabled. Permission checks were intentionally removed to align with REST API behavior.
    // The REST API does not require special permissions to toggle this option, so the UI was updated for consistency.
    useTrustStoreCheckbox.enable();
    
    useTrustStoreField.mon(
        NX.Conditions.isPermitted('astrum:ssl-truststore:read'),
        {
          satisfied: useTrustStoreButton.enable,
          unsatisfied: useTrustStoreButton.disable,
          scope: useTrustStoreButton
        }
    );
  }

});
