
/*global Ext, AS*/

/**
 * LDAP controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.LdapServers', {
  extend: 'AS.controller.Drilldown',
  requires: [
    'AS.Conditions',
    'AS.Messages',
    'AS.Permissions',
    'AS.I18n'
  ],
  masters: [
    'as-coreui-ldapserver-list'
  ],
  models: [
    'LdapServer'
  ],
  stores: [
    'LdapServer',
    'LdapSchemaTemplate'
  ],
  views: [
    'ldap.LdapServerChangeOrder',
    'ldap.LdapServerFeature',
    'ldap.LdapServerList',
    'ldap.LdapServerConnection',
    'ldap.LdapServerConnectionFieldSet',
    'ldap.LdapServerConnectionForm',
    'ldap.LdapServerUserAndGroup',
    'ldap.LdapServerUserAndGroupFieldSet',
    'ldap.LdapServerUserAndGroupForm',
    'ldap.LdapServerUserAndGroupLoginCredentials',
    'ldap.LdapServerUserAndGroupMappingTestResults',
    'ldap.LdapServerConnectionAdd',
    'ldap.LdapServerUserAndGroupAdd',
    'ldap.LdapSystemPasswordModal'
  ],
  refs: [
    { ref: 'main', selector: '#feature-content' },
    { ref: 'feature', selector: 'as-coreui-ldapserver-feature' },
    { ref: 'content', selector: 'as-feature-content' },
    { ref: 'list', selector: 'as-coreui-ldapserver-list' },
    { ref: 'connection', selector: 'as-coreui-ldapserver-feature as-coreui-ldapserver-connection' },
    { ref: 'userAndGroup', selector: 'as-coreui-ldapserver-feature as-coreui-ldapserver-userandgroup' }
  ],
  icons: {
    'ldapserver-default': {
      file: 'book_addresses.png',
      variants: ['x16', 'x32']
    },
    'ldap-security-source': {
      file: 'book_addresses.png',
      variants: ['x16']
    }
  },

  permission: 'astrum:ldap',

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.callParent();

    me.listen({
      controller: {
        '#Refresh': {
          refresh: me.loadStores
        }
      },
      store: {
        '#LdapServer': {
          load: me.reselect
        }
      },
      component: {
        'as-coreui-ldapserver-list': {
          beforerender: me.loadStores
        },
        'as-coreui-ldapserver-list button[action=new]': {
          click: me.showConnectionPanel
        },
        'as-coreui-ldapserver-connection-add button[action=next]': {
          click: me.showUserAndGroupPanel
        },
        'as-coreui-ldapserver-userandgroup-form button[action=add]': {
          click: me.createServer
        },
        'as-coreui-ldapserver-connection-form button[action=save]': {
          click: me.updateServer
        },
        'as-coreui-ldapserver-userandgroup-form button[action=save]': {
          click: me.updateServer
        },
        'as-coreui-ldapserver-list button[action=changeorder]': {
          click: me.showChangeOrder,
          afterrender: me.bindChangeOrderButton
        },
        'as-coreui-ldapserver-list button[action=clearcache]': {
          click: me.clearCache,
          afterrender: me.bindClearCacheButton
        },
        'as-coreui-ldapserver-changeorder button[action=save]': {
          click: me.changeOrder
        },
        'as-coreui-ldapserver-connection-add button[action=verifyconnection]': {
          click: me.verifyConnection
        },
        'as-coreui-ldapserver-connection button[action=verifyconnection]': {
          click: me.verifyConnection
        },
        'as-coreui-ldapserver-connection-fieldset button[action=setpassword]': {
          click: me.updatePass
        },
        'as-coreui-ldapserver-userandgroup-add button[action=verifyusermapping]': {
          click: me.verifyUserMapping
        },
        'as-coreui-ldapserver-userandgroup button[action=verifyusermapping]': {
          click: me.verifyUserMapping
        },
        'as-coreui-ldapserver-userandgroup-add button[action=verifylogin]': {
          click: me.showLoginCredentialsWindow
        },
        'as-coreui-ldapserver-userandgroup button[action=verifylogin]': {
          click: me.showLoginCredentialsWindow
        },
        'as-coreui-ldapserver-userandgroup-login-credentials button[action=verifylogin]': {
          click: me.verifyLogin
        },
        'as-coreui-ldapserver-systempassword-modal button[action=ok]': {
          click: me.ldapSystemPasswordCollected
        }
      }
    });
  },

  /**
   * @override
   */
  getDescription: function(model) {
    return model.get('name');
  },

  /**
   * @override
   */
  onSelection: function(list, model) {
    var me = this;

    if (Ext.isDefined(model)) {
      Ext.suspendLayouts();

      me.connectionInfo = model.getData();
      me.getConnection().loadRecord(model);
      me.getUserAndGroup().loadRecord(model);
      me.getUserAndGroup().down('#template').setValue(null);

      Ext.resumeLayouts(true);
    }
  },

  /**
   * @private
   */
  showConnectionPanel: function() {
    var me = this;

    me.connectionInfo = {};

    // Show the first panel in the create wizard, and set the breadcrumb
    me.setItemName(1, AS.I18n.get('LdapServers_CreateConnection_Title'));
    me.loadCreateWizard(1, Ext.widget({
      xtype: 'panel',
      layout: {
        type: 'vbox',
        align: 'stretch',
        pack: 'start'
      },
      items: [
        {
          xtype: 'as-coreui-ldapserver-connection-add',
          flex: 1
        }
      ]
    }));
  },

  /**
   * @private
   */
  showUserAndGroupPanel: function() {
    var me = this;

    me.connectionInfo = me.getFeature().down('as-coreui-ldapserver-connection-add').down('as-coreui-ldapserver-connection-form').getForm().getValues();

    // Show the first panel in the create wizard, and set the breadcrumb
    me.setItemName(2, AS.I18n.get('LdapServers_CreateUsersAndGroups_Title'));
    me.loadCreateWizard(2, Ext.widget({
      xtype: 'panel',
      layout: {
        type: 'vbox',
        align: 'stretch',
        pack: 'start'
      },
      items: [
        {
          xtype: 'as-coreui-ldapserver-userandgroup-add',
          flex: 1
        }
      ]
    }));
  },

  /**
   * @private
   */
  showChangeOrder: function() {
    Ext.widget('as-coreui-ldapserver-changeorder');
  },

  /**
   * @private
   * Update an existing LDAP entry with new pass
   */
  updatePass: function() {
    const me = this,
        values = me.getValues();

    //all options except anonymous (none) require a password, so throw up the dialog if not set in form
    if (values.authScheme !== 'none') {
      Ext.create('AS.coreui.view.ldap.LdapSystemPasswordModal', {
        onSuccess: function(pass) {
          me.doVerifyConnection(true, pass);
        }
      });
    }
    else {
      this.doVerifyConnection(true);
    }
  },

  /**
   * @private
   * Update an existing LDAP entry
   */
  updateServer: function() {
    const me = this,
          values = me.getValues();

    //all options except anonymous (none) require a password, so throw up the dialog if not set in form
    if (values.authScheme !== 'none' && !values.authPassword) {
      Ext.create('AS.coreui.view.ldap.LdapSystemPasswordModal', {
        onSuccess: function(pass) {
          me.doVerifyConnection(true, pass);
        }
      });
    }
    else {
      this.doVerifyConnection(true);
    }
  },

  doUpdateServer: function(ldapSystemPassword) {
    const me = this,
          feature = me.getFeature(),
          connectionForm = feature.down('as-coreui-ldapserver-connection').down('as-coreui-ldapserver-connection-form'),
          userGroupForm = feature.down('as-coreui-ldapserver-userandgroup').down('as-coreui-ldapserver-userandgroup-form'),
          values = {};

    // Get fields from all relevant forms
    Ext.apply(values, connectionForm.getForm().getFieldValues());
    Ext.apply(values, userGroupForm.getForm().getFieldValues());
    if (ldapSystemPassword) {
      values.authPassword = ldapSystemPassword;
    }
    const modelData = connectionForm.getForm().getRecord().getData(false);

    Object.keys(values).forEach(function(field) {
      delete modelData[field];
    });
    Ext.apply(modelData, values);

    // Apply masks
    me.getContent().getEl().mask(AS.I18n.get('LdapServers_Update_Mask'));
    AS.direct.ldap_LdapServer.update(modelData, function(response) {
      me.getContent().getEl().unmask();
      if (Ext.isObject(response)) {
        if (response.success) {
          AS.Messages.success(AS.I18n.format('LdapServers_Update_Success',
              me.getDescription(me.getLdapServerModel().create(response.data))));
          me.getStore('LdapServer').load();
        }
      }
    });
  },

  /**
   * @private
   * Create a new LDAP entry
   */
  createServer: function() {
    var me = this,
      feature = me.getFeature(),
      userGroupForm = feature.down('as-coreui-ldapserver-userandgroup-add').down('as-coreui-ldapserver-userandgroup-form'),
      values = me.connectionInfo;

    // Get fields from all relevant forms
    Ext.apply(values, userGroupForm.getForm().getFieldValues());

    me.getContent().getEl().mask(AS.I18n.get('LdapServers_Create_Mask'));
    AS.direct.ldap_LdapServer.create(values, function(response) {
      me.getContent().getEl().unmask();
      if (Ext.isObject(response)) {
        if (response.success) {
          AS.Messages.success(AS.I18n.format('LdapServers_Create_Success',
              me.getDescription(me.getLdapServerModel().create(response.data))));
          me.getStore('LdapServer').load();
        }
      }
    });
  },

  /**
   * @private
   * Enable 'Change Order' when user has 'update' permission.
   */
  bindChangeOrderButton: function(button) {
    button.mon(
        AS.Conditions.isPermitted(this.permission + ':update'),
        {
          satisfied: function() {
            button.enable();
          },
          unsatisfied: function() {
            button.disable();
          }
        }
    );
  },

  /**
   * @private
   * Enable 'ClearCache' when user has 'delete' permission and there is at least one LDAP server configured.
   */
  bindClearCacheButton: function(button) {
    button.mon(
        AS.Conditions.and(
            AS.Conditions.isPermitted(this.permission + ':delete'),
            AS.Conditions.storeHasRecords('LdapServer')
        ),
        {
          satisfied: function () {
            button.enable();
          },
          unsatisfied: function () {
            button.disable();
          }
        }
    );
  },

  /**
   * @private
   * @override
   * Deletes a LDAP server.
   * @param {AS.coreui.model.LdapServer} model to be deleted
   */
  deleteModel: function(model) {
    var me = this,
        description = me.getDescription(model);

    AS.direct.ldap_LdapServer.remove(model.getId(), function(response) {
      me.getStore('LdapServer').load();
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.format('LdapServers_Delete_Success', description));
      }
    });
  },

  /**
   * @private
   * Change LDAP servers order.
   */
  changeOrder: function(button) {
    var me = this,
        win = button.up('window'),
        order = button.up('form').down('as-itemorderer').getValue();

    AS.direct.ldap_LdapServer.changeOrder(order, function(response) {
      if (Ext.isObject(response) && response.success) {
        win.close();
        AS.Messages.success(AS.I18n.get('LdapServers_ChangeOrder_Success'));
        me.getStore('LdapServer').load();
      }
    });
  },

  /**
   * @private
   * Clear LDAP cache.
   */
  clearCache: function(button) {
    AS.direct.ldap_LdapServer.clearCache(function(response) {
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.get('LdapServers_ClearCache_Success'));
      }
    });
  },

  /**
   * @private
   * Verify LDAP server connection.
   */
  verifyConnection: function(button) {
    const me = this,
        form = button.up('form'),
        values = form.getForm().getFieldValues();

    //all options except anonymous (none) require a password, so throw up the dialog if not set in form
    if (values.authScheme !== 'none' && !values.authPassword) {
      Ext.create('AS.coreui.view.ldap.LdapSystemPasswordModal', {
        onSuccess: function(pass) {
          me.doVerifyConnection(false, pass);
        }
      });
    }
    else {
      this.doVerifyConnection(false);
    }
  },

  /**
   * @private
   * Update the ldap system password in the form so it will be used in requests
   */
  ldapSystemPasswordCollected: function(button) {
    const modal = button.up('as-coreui-ldapserver-systempassword-modal'),
        ldapSystemPassword = modal.down('as-password').getValue();

    modal.close();

    if (modal.onSuccess) {
      modal.onSuccess.call(this, ldapSystemPassword);
    }
  },

  /**
   * @private
   * Verify LDAP user mapping.
   */
  verifyUserMapping: function() {
    const me = this,
        values = me.getValues();

    //all options except anonymous (none) require a password, so throw up the dialog if not set in form
    if (values.authScheme !== 'none' && !values.authPassword) {
      Ext.create('AS.coreui.view.ldap.LdapSystemPasswordModal', {
        onSuccess: me.doVerifyUserMapping
      });
    }
    else {
      me.doVerifyUserMapping();
    }
  },

  doVerifyUserMapping: function(ldapSystemPassword) {
    const me = this,
        values = me.getValues(),
        url = values.protocol + '://' + values.host + ':' + values.port;

    if (ldapSystemPassword) {
      values.authPassword = ldapSystemPassword;
    }

    me.getMain().getEl().mask(AS.I18n.format('LdapServers_VerifyMapping_Mask', url));

    AS.direct.ldap_LdapServer.verifyUserMapping(values, function(response) {
      me.getMain().getEl().unmask();
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.format('LdapServers_VerifyMapping_Success', url));
        Ext.widget('as-coreui-ldapserver-userandgroup-testresults', {mappedUsers: response.data});
      }
    });
  },

  doVerifyConnection: function(shouldUpdate, ldapSystemPassword) {
    const me = this,
          values = me.getValues(),
          url = values.protocol + '://' + values.host + ':' + values.port;

    if (ldapSystemPassword) {
      values.authPassword = ldapSystemPassword;
    }
    me.getMain().getEl().mask(AS.I18n.format('LdapServers_VerifyConnection_Mask', url));

    AS.direct.ldap_LdapServer.verifyConnection(values, function(response) {
      me.getMain().getEl().unmask();
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.format('LdapServers_VerifyConnection_Success', url));
        if (shouldUpdate) {
          me.doUpdateServer(values.authPassword);
        }
      }
      else if (Ext.isObject(response) && Ext.isDefined(response.errors)) {
        AS.Messages.error(response.errors['*']);
      }
    });
  },

  /**
   * @private
   */
  showLoginCredentialsWindow: function() {
    const me = this,
        values = me.getValues();

    //all options except anonymous (none) require a password, so throw up the dialog if not set in form
    if (values.authScheme !== 'none' && !values.authPassword) {
      Ext.create('AS.coreui.view.ldap.LdapSystemPasswordModal', {
        onSuccess: me.doShowLoginCredentialsWindow
      });
    }
    else {
      me.doShowLoginCredentialsWindow();
    }
  },

  /**
   * @private
   */
  doShowLoginCredentialsWindow: function(ldapSystemPassword) {
    Ext.widget('as-coreui-ldapserver-userandgroup-login-credentials', {
      ldapSystemPassword: ldapSystemPassword
    });
  },

  /**
   * @private
   * Verify LDAP login.
   */
  verifyLogin: function(button) {
    var win = button.up('window'),
        form = button.up('form'),
        cmp = button.up('as-coreui-ldapserver-userandgroup-login-credentials'),
        loginValues = form.getForm().getFieldValues(),
        userName = AS.util.Base64.encode(loginValues.username),
        userPass = AS.util.Base64.encode(loginValues.password),
        values = this.getValues(),
        url = values.protocol + '://' + values.host + ':' + values.port;

    if (cmp.ldapSystemPassword) {
      values.authPassword = cmp.ldapSystemPassword;
    }

    form.getEl().mask(AS.I18n.format('LdapServers_VerifyLogin_Mask', url));

    AS.direct.ldap_LdapServer.verifyLogin(values, userName, userPass, function(response) {
      form.getEl().unmask();
      if (Ext.isObject(response) && response.success) {
        win.close();
        AS.Messages.success(AS.I18n.format('LdapServers_VerifyLogin_Success', url));
      }
    });
  },

  /**
   * @private
   * Get form values from connection adn user/group form.
   */
  getValues: function() {
    var feature = this.getFeature(),
        values = this.connectionInfo,
        url, connectionForm, userGroupForm;

    if (feature.down('as-coreui-ldapserver-connection-add')) {
      connectionForm = feature.down('as-coreui-ldapserver-connection-add').down('as-coreui-ldapserver-connection-form');
    }
    else {
      connectionForm = feature.down('as-coreui-ldapserver-connection').down('as-coreui-ldapserver-connection-form');
    }
    if (feature.down('as-coreui-ldapserver-userandgroup-add')) {
      userGroupForm = feature.down('as-coreui-ldapserver-userandgroup-add').down('as-coreui-ldapserver-userandgroup-form');
    }
    else {
      userGroupForm = feature.down('as-coreui-ldapserver-userandgroup').down('as-coreui-ldapserver-userandgroup-form');
    }

    // Get fields from all relevant forms
    Ext.apply(values, connectionForm.getForm().getFieldValues());
    Ext.apply(values, userGroupForm.getForm().getFieldValues());

    return values;
  },

  onNavigate: function() {
    var bookmark = AS.Bookmarks.getBookmark();
    var token = bookmark && bookmark.getToken ? bookmark.getToken() : null;
    if (token && token.includes('security/ldap')) {
      this.reselect();
    }
  }

});
