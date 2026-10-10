
/*global Ext, AS*/

/**
 * Ssl Certificates controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.SslCertificates', {
  extend: 'AS.controller.Drilldown',
  requires: [
    'AS.Messages',
    'AS.Permissions',
    'AS.I18n'
  ],
  masters: [
    'as-coreui-sslcertificate-list'
  ],
  models: [
    'SslCertificate'
  ],
  stores: [
    'SslCertificate'
  ],
  views: [
    'ssl.SslCertificateAddFromPem',
    'ssl.SslCertificateAddFromServer',
    'ssl.SslCertificateDetails',
    'ssl.SslCertificateDetailsForm',
    'ssl.SslCertificateDetailsPanel',
    'ssl.SslCertificateDetailsWindow',
    'ssl.SslCertificateFeature',
    'ssl.SslCertificateList'
  ],
  refs: [
    { ref: 'feature', selector: 'as-coreui-sslcertificate-feature' },
    { ref: 'content', selector: 'as-feature-content' },
    { ref: 'list', selector: 'as-coreui-sslcertificate-list' },
    { ref: 'details', selector: 'as-coreui-sslcertificate-feature as-coreui-sslcertificate-details-form' }
  ],
  icons: {
    'sslcertificate-default': {
      file: 'ssl_certificates.png',
      variants: ['x16', 'x32']
    },
    'sslcertificate-add-by-pem': {
      file: 'server_add.png',
      variants: ['x16', 'x32']
    },
    'sslcertificate-add-by-server': {
      file: 'server_connect.png',
      variants: ['x16', 'x32']
    }
  },

  permission: 'astrum:ssl-truststore',

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.callParent();

    me.listen({
      controller: {
        '#Refresh': {
          refresh: me.loadStores
        }
      },
      store: {
        '#SslCertificate': {
          load: me.reselect
        }
      },
      component: {
        'as-coreui-sslcertificate-list menuitem[action=newfromserver]': {
          click: me.showAddWindowFromServer
        },
        'as-coreui-sslcertificate-list menuitem[action=newfrompem]': {
          click: me.showAddWindowFromPem
        },
        'as-coreui-sslcertificate-add-from-pem button[action=load]': {
          click: me.loadCertificateByPem
        },
        'as-coreui-sslcertificate-add-from-server button[action=load]': {
          click: me.loadCertificateByServer
        },
        'as-coreui-sslcertificate-details-panel button[action=add]': {
          click: me.create
        },
        'as-coreui-sslcertificate-details-panel button[action=remove]': {
          click: me.onRemoveClick
        },
        'as-coreui-sslcertificate-details-window button[action=add]': {
          click: me.create
        },
        'as-coreui-sslcertificate-details-window button[action=remove]': {
          click: me.onRemoveClick
        }
      }
    });
  },

  /**
   * @override
   */
  getDescription: function (model) {
    return model.get('subjectCommonName');
  },

  /**
   * @override
   */
  onSelection: function (list, model) {
    if (Ext.isDefined(model)) {
      this.getDetails().loadRecord(model);
    }
  },

  /**
   * @private
   */
  showAddWindowFromServer: function () {
    var me = this;

    // Show the first panel in the create wizard, and set the breadcrumb
    me.setItemName(1, AS.I18n.get('SslCertificates_Load_Title'));
    me.loadCreateWizard(1, Ext.create('widget.as-coreui-sslcertificate-add-from-server'));
  },

  /**
   * @private
   */
  showAddWindowFromPem: function () {
    var me = this;

    // Show the first panel in the create wizard, and set the breadcrumb
    me.setItemName(1, AS.I18n.get('SslCertificates_Paste_Title'));
    me.loadCreateWizard(1, Ext.create('widget.as-coreui-sslcertificate-add-from-pem'));
  },

  /**
   * @private
   * Shows certificate details panel.
   */
  showCertificateDetailsPanel: function(certificate) {
    var me = this,
        panel = Ext.widget('as-coreui-sslcertificate-details-panel'),
        form = panel.down('form'),
        model = me.getSslCertificateModel().create(certificate);

    // override form to show buttons
    me.overrideLoadRecord(form);
    // Load the certificate
    form.loadRecord(model);

    // Show the second panel in the create wiard, and set the breadcrumb
    me.setItemName(2, AS.I18n.get('Ssl_SslCertificateDetailsWindow_Title'));
    me.loadCreateWizard(2, panel);
  },

  /**
   * @private
   * Shows certificate details window.
   */
  showCertificateDetailsWindow: function (certificate) {
    var me = this,
        window = Ext.widget('as-coreui-sslcertificate-details-window'),
        form = window.down('form'),
        model = me.getSslCertificateModel().create(certificate);

    // override form to show buttons
    me.overrideLoadRecord(form);
    // Load the certificate
    form.loadRecord(model);
  },

  /**
   * @private
   * Retrieves details of a certificate specified by PEM, showing the certificate details if successful.
   */
  loadCertificateByPem: function (button) {
    var me = this,
        basicForm = button.up('form').getForm(),
        pem = basicForm.getFieldValues()['pem'];

    me.getContent().getEl().mask(AS.I18n.get('SslTrustStore_Load_Mask'));
    AS.direct.ssl_Certificate.details(pem, function(response) {
      me.getContent().getEl().unmask();
      if (Ext.isObject(response)) {
        if (response.success) {
          me.showCertificateDetailsPanel(response.data);
        }
        else if (response.errors) {
          basicForm.markInvalid(response.errors);
        }
      }
    });
  },

  /**
   * @private
   * Retrieves certificate from host, showing the certificate details if successful.
   */
  loadCertificateByServer: function (button) {
    var me = this,
        server = button.up('form').getForm().getFieldValues()['server'],
        parsed = me.parseHostAndPort(server),
        protocolHint = server && Ext.String.startsWith(server, "https://") ? 'https' : undefined;

    me.getContent().getEl().mask(AS.I18n.get('SslTrustStore_Load_Mask'));
    AS.direct.ssl_Certificate.retrieveFromHost(parsed[0], parsed[1], protocolHint, function (response) {
      me.getContent().getEl().unmask();
      if (Ext.isObject(response)) {
        if (response.success) {
          me.showCertificateDetailsPanel(response.data);
        }
        else if (response.errors) {
          AS.Messages.error(Object.values(response.errors));
        }
      }
    });
  },

  /**
   * @private
   * Parses server string into an array of [host,port].
   */
  parseHostAndPort: function (server) {
    var rx, matches;

    if (!server) {
      return [];
    }

    // neither URL nor host:port
    if (server.indexOf(":") === -1) {
      return [server];
    }

    // definitely no URL
    if (server.indexOf("/") === -1) {
      var serverPort = server.split(":");

      if (serverPort.length === 2) {
        serverPort[1] = Ext.Number.from(serverPort[1], undefined);
      }

      return serverPort;
    }

    rx = /[^:]*:\/\/([^:\/]*)(:([0-9]*))?/;
    matches = server.match(rx);
    if (matches) {
      return [matches[1], Ext.Number.from(matches[3], undefined)];
    }

    return [server];
  },

  /**
   * @private
   * Close the SSL certificate details window
   * that contains the button, if it exists.
   * @param {Ext.button.Button} button (optional)
   */
  closeCertDetailsWindowIfExists: function (button) {
    if (button) {
      var window = button.up('as-coreui-sslcertificate-details-window');
      if (window) {
        window.close();
      }
    }
  },

  /**
   * @private
   * Creates an SSL certificate.
   */
  create: function (button) {
    var me = this,
        form = button.up('form'),
        model = form.getRecord(),
        description = me.getDescription(model);

    AS.direct.ssl_TrustStore.create(model.get('pem'), function (response) {
      me.getStore('SslCertificate').load();
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.format('SslCertificates_Load_Success', description));
        me.closeCertDetailsWindowIfExists(button);
      }
    });
  },

  /**
   * @private
   * Deletes an SSL certificate
   */
  onRemoveClick: function (button) {
    var form = button.up('form'),
        model = form.getRecord();

    this.deleteModel(model, button);
  },

  /**
   * @private
   * @override
   * Deletes an SSL certificate.
   * @param {AS.coreui.model.SslCertificate} model to be deleted
   * @param {Ext.button.Button} button (optional)
   */
  deleteModel: function (model, button) {
    var me = this,
        description = me.getDescription(model);

    AS.direct.ssl_TrustStore.remove(model.getId(), function (response) {
      me.getStore('SslCertificate').load();
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.format('SslCertificates_Delete_Success', description));
        me.closeCertDetailsWindowIfExists(button);
      }
    });
  },

  /**
   * @private
   * Override loadRecord() in order to show add/delete from store buttons.
   */
  overrideLoadRecord: function (form) {
    Ext.override(form, {
      loadRecord: function (model) {
        var me = this,
            tbar = me.getDockedItems('toolbar[dock="bottom"]')[0],
            button;

        if (model) {
          if (model.get('inTrustStore')) {
            tbar.insert(0, {
              text: AS.I18n.get('SslCertificates_Remove_Button'),
              action: 'remove',
              formBind: true,
              disabled: true,
              ui: 'as-primary',
              iconCls: 'x-fa fa-trash'
            });
            button = tbar.down('button[action=remove]');
            me.mon(
                AS.Conditions.isPermitted('astrum:ssl-truststore:delete'),
                {
                  satisfied: function () {
                    button.enable();
                  },
                  unsatisfied: function () {
                    button.disable();
                  }
                }
            );
          }
          else {
            tbar.insert(0, {
              text: AS.I18n.get('SslCertificates_Add_Button'),
              action: 'add',
              formBind: true,
              disabled: true,
              ui: 'as-primary',
              iconCls: 'x-fa fa-plus-circle'
            });
            button = tbar.down('button[action=add]');
            me.mon(
                AS.Conditions.isPermitted('astrum:ssl-truststore:create'),
                {
                  satisfied: function () {
                    button.enable();
                  },
                  unsatisfied: function () {
                    button.disable();
                  }
                }
            );
          }
        }
        me.callParent(arguments);
      }
    });
  }

});
