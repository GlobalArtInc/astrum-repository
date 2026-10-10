
/*global Ext, AS*/

/**
 * Users controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.Users', {
  extend: 'AS.controller.Drilldown',
  requires: [
    'AS.Conditions',
    'AS.State',
    'AS.Permissions',
    'AS.Security',
    'AS.Icons',
    'AS.Messages',
    'AS.Dialogs',
    'AS.I18n',
    'AS.util.Filter',
    'Ext.History'
  ],
  masters: [
    'as-coreui-user-list'
  ],
  models: [
    'User'
  ],
  stores: [
    'User',
    'UserSource',
    'Role'
  ],
  views: [
    'user.UserAdd',
    'user.UserChangePassword',
    'user.UserFeature',
    'user.UserList',
    'user.UserSearchBox',
    'user.UserSettings',
    'user.UserSettingsForm',
    'user.UserSettingsExternal',
    'user.UserSettingsExternalForm'
  ],
  refs: [
    { ref: 'feature', selector: 'as-coreui-user-feature' },
    { ref: 'list', selector: 'as-coreui-user-list' },
    { ref: 'userSearchBox', selector: 'as-coreui-user-list as-coreui-user-searchbox' },
    { ref: 'settings', selector: 'as-coreui-user-feature as-coreui-user-settings' },
    { ref: 'externalSettings', selector: 'as-coreui-user-feature as-coreui-user-settings-external' }
  ],
  icons: {
    'user-default': {
      file: 'user.png',
      variants: ['x16', 'x32']
    },
    'default-security-source': {
      file: 'user.png',
      variants: ['x16']
    },
    'allconfigured-security-source': {
      file: 'user.png',
      variants: ['x16']
    },
    'all-security-source': {
      file: 'group.png',
      variants: ['x16']
    },
    'oauth2-security-source': {
      file: 'user.png',
      variants: ['x16']
    }
  },

  permission: 'astrum:users',

  /**
   * @override
   */
  init: function() {
    var me = this;

    me.features = [
      {
        mode: 'admin',
        path: '/Security/Users',
        text: AS.I18n.get('User_Text'),
        description: AS.I18n.get('User_Description'),
        view: {xtype: 'as-coreui-user-feature'},
        iconConfig: {
          file: 'group.png',
          variants: ['x16', 'x32']
        },
        visible: function() {
          return AS.Permissions.check('astrum:users:read') &&
              AS.Permissions.check('astrum:roles:read') &&
              !AS.State.getValue('astrum.react.users', false);
        },
        weight: 30
      },
    ];

    me.callParent();

    me.listen({
      controller: {
        '#Refresh': {
          refresh: me.loadStores
        },
        '*': {
          userSettingsAddTab: me.addTab,
          userSettingsRemoveTab: me.removeTab
        }
      },
      store: {
        '#User': {
          load: me.reselect
        },
        '#UserSource': {
          load: me.onUserSourceLoad
        }
      },
      component: {
        'as-coreui-user-list': {
          beforerender: me.loadStores
        },
        'as-coreui-user-list button[action=new]': {
          click: me.showAddWindow
        },
        'as-coreui-user-settings-form': {
          submitted: me.onSettingsSubmitted
        },
        'as-coreui-user-settings-external-form': {
          submitted: me.onSettingsSubmitted
        },
        'as-coreui-user-list menuitem[action=filter]': {
          click: me.onSourceChanged
        },
        'as-coreui-user-feature button[action=setpassword]': {
          afterrender: me.bindSetPasswordButton,
          click: me.showChangePasswordWindowForSelection
        },
        'as-coreui-user-list as-coreui-user-searchbox': {
          search: me.loadStore,
          searchcleared: me.onSearchCleared
        },
        'as-coreui-user-account button[action=changepassword]': {
          click: me.showChangePasswordWindowForUserAccount,
          afterrender: me.bindChangePasswordButton
        },
        'as-coreui-user-changepassword button[action=changepassword]': {
          click: me.changePassword
        }
      }
    });
  },

  /**
   * @override
   */
  getDescription: function(model) {
    return model.get('firstName') + ' ' + model.get('lastName');
  },

  /**
   * @override
   */
  onSelection: function(list, model) {
    var me = this,
        settingsPanel = me.getSettings(),
        externalSettingsPanel = me.getExternalSettings();

    if (Ext.isDefined(model)) {
      if (model.get('external')) {
        if (!externalSettingsPanel) {
          me.addTab({ xtype: 'as-coreui-user-settings-external', title: AS.I18n.get('Users_Settings_Title'), weight: 10 });
          externalSettingsPanel = me.getExternalSettings();
        }
        //we purposefully don't load the roles/externalRoles when retrieving list as we need to query crowd for each user
        //very time consuming, rather we query for the role data when a user is selected
        AS.direct.coreui_User.get(model.get('userId'), model.get('realm'), function(response) {
          if (Ext.isObject(response) && response.success) {
              model.set('roles', response.data.roles);
              model.set('externalRoles', response.data.externalRoles);
              externalSettingsPanel.loadRecord(model);
          }
          else {
            me.removeTab(externalSettingsPanel);
          }
        });
      }
      else {
        if (!settingsPanel) {
          me.addTab({ xtype: 'as-coreui-user-settings', title: AS.I18n.get('Users_Settings_Title'), weight: 10 });
          settingsPanel = me.getSettings();
        }
        settingsPanel.loadRecord(model);
      }

      if (model.get('external')) {
        if (settingsPanel) {
          me.removeTab(settingsPanel);
        }
      }
      else {
        if (externalSettingsPanel) {
          me.removeTab(externalSettingsPanel);
        }
      }
    }
  },

  /**
   * @private
   */
  showAddWindow: function() {
    var me = this;

    // Show the first panel in the create wizard, and set the breadcrumb
    me.setItemName(1, AS.I18n.get('Users_Create_Title'));
    me.loadCreateWizard(1, Ext.create('widget.as-coreui-user-add'));
  },

  /**
   * @private
   */
  showChangePasswordWindowForSelection: function() {
    var list = this.getList(),
        userId = list.getSelectionModel().getSelection()[0].get('userId');

    Ext.widget('as-coreui-user-changepassword', { userId: userId });
  },

  /**
   * @private
   */
  showChangePasswordWindowForUserAccount: function(button) {
    var userId = button.up('form').down('#userId').getValue();

    Ext.widget('as-coreui-user-changepassword', { userId: userId });
  },

  /**
   * @override
   * Override in order to filter users by selected source (user manager) and user id.
   */
  loadStore: function(cb) {
    var me = this,
        list = me.getList(),
        userSourceButton;

    if (list) {
      userSourceButton = list.down('button[action=filter]');
      if (!userSourceButton.sourceId) {
        userSourceButton.sourceId = 'all';
      }
      me.updateEmptyText();
      me.getStore('User').load({
        params: {
          filter: [
            { property: 'source', value: userSourceButton.sourceId },
            { property: 'userId', value: me.getUserSearchBox().getValue() }
          ]
        },
        callback: function(records, operation, success) {
          if (Ext.isFunction(cb)) {
            cb(records, operation, success);
          }
        }
      });
    }
  },

  /**
   * @override
   * Load all of the stores associated with this controller
   */
  loadStores: function() {
    var me = this,
        list = me.getList();

    if (list) {
      me.loadStore();
      me.getStore('UserSource').load();
      me.getStore('Role').load();
    }
  },

  /**
   * @private
   * (Re)create user source filters.
   */
  onUserSourceLoad: function(store) {
    var list = this.getList(),
        userSourceButton;

    if (list) {
      userSourceButton = list.down('button[action=filter]');
      if (userSourceButton.menu.items.length > 1) {
        userSourceButton.menu.removeAll();
      }
      if (!userSourceButton.sourceId) {
        userSourceButton.sourceId = 'all';
      }
      store.each(function(source) {
        var iconCls = AS.Icons.cls(source.getId().toLowerCase() + '-security-source', 'x16');
        userSourceButton.menu.add({
          text: source.get('name'),
          iconCls: iconCls,
          group: 'usersource',
          checked: userSourceButton.sourceId === source.getId(),
          action: 'filter',
          source: source
        });
        if (userSourceButton.sourceId === source.getId()) {
          userSourceButton.setText(source.get('name'));
          userSourceButton.setIconCls(iconCls);
        }
      });
    }
  },

  /**
   * @private
   */
  onSearchCleared: function() {
    var me = this,
        list = me.getList(),
        userSourceButton = list.down('button[action=filter]');

    if (userSourceButton.sourceId === 'default' || userSourceButton.sourceId === 'all') {
      me.loadStore();
    }
    else {
      me.updateEmptyText();
      me.getStore('User').removeAll();
    }
  },

  /**
   * @private
   */
  onSourceChanged: function(menuItem) {
    var me = this,
        list = me.getList(),
        userSourceButton = list.down('button[action=filter]');

    userSourceButton.setText(menuItem.source.get('name'));
    userSourceButton.setIconCls(menuItem.iconCls);
    userSourceButton.sourceId = menuItem.source.getId();

    me.getUserSearchBox().setValue(undefined);
    me.loadStore();
  },

  /**
   * @private
   * Update grid empty text based on source/user id from search box.
   */
  updateEmptyText: function() {
    var me = this,
        list = me.getList(),
        userSourceButton = list.down('button[action=filter]'),
        userId = me.getUserSearchBox().getValue(),
        emptyText = '';

    if (userSourceButton.sourceId === 'default' || userSourceButton.sourceId === 'all') {
      if (userId) {
        emptyText += AS.I18n.get('Users_Filter_NoMatch_EmptyText');
      }
      else {
        emptyText += AS.I18n.get('Users_Filter_Empty_EmptyText');
      }
    }
    else {
      emptyText += 'No ' + userSourceButton.getText() + ' user matched query criteria';
      if (userId) {
        emptyText += ' "$filter"';
      }
    }

    list.getView().emptyText = AS.util.Filter.buildEmptyResult(userId, emptyText);
  },

  /**
   * @private
   */
  onSettingsSubmitted: function(form, action) {
    var me = this,
        list = me.getList(),
        selection = list.getSelection();

    form.getForm().getFields().each(function(field) {
      field.resetOriginalValue();
    });

    me.suppressReselect = true;
    me.loadStore(function(records) {
      me.suppressReselect = false;
      var selectedId, found = false;
      if (selection && selection.length > 0) {
        selectedId = selection[0].get('userId');
        records.forEach(function(candidate) {
          if (candidate.get('userId') === selectedId) {
            list.setSelection(candidate);
            found = true;
          }
        });
        if (!found) {
          var model = selection[0];
          var externalSettingsPanel = me.getExternalSettings();
          if (model.get('external') && externalSettingsPanel) {
            AS.direct.coreui_User.get(model.get('userId'), model.get('realm'), function(response) {
              if (Ext.isObject(response) && response.success) {
                model.set('roles', response.data.roles);
                model.set('externalRoles', response.data.externalRoles);
                externalSettingsPanel.loadRecord(model);
              }
              else {
                me.navigateTo(AS.Bookmarks.fromToken('admin/security/users'));
              }
            });
          }
          else {
            var settingsPanel = me.getSettings();
            if (settingsPanel) {
              settingsPanel.loadRecord(model);
            }
          }
        }
      }
    });
  },

  /**
   * @private
   */
  watchEventsHandler: function (skipCurrentUser) {
    var me = this,
        store = me.getStore('User');

    return function() {
      var userId = me.getModelIdFromBookmark(),
          enableButton = false,
          selectedModel;

      if (userId) {
        selectedModel = store.findRecord('userId', userId, 0, false, true, true);

        // Allow SAML and OAuth2 users to be deleted because we create copies in the astrum DB when
        // such users authenticate, the customer may want to clean these up.
        // LDAP user-info is never copied to our DB, so the delete button should still be hidden
        // for LDAP users
        enableButton = selectedModel
            && (!selectedModel.get('external')
                || (selectedModel.get('realm') === "SAML" && skipCurrentUser !== undefined)
                || (selectedModel.get('realm') === "OAuth2" && skipCurrentUser !== undefined)
            )
            && (selectedModel.get('userId') !== AS.State.getValue('anonymousUsername'));

        if (enableButton && skipCurrentUser) {
          // AS.State.getUser() is undefined for anonymous subjects; treat "no current user"
          // as "not the current user" so the button remains enabled (ASTRUM-47114).
          var currentUser = AS.State.getUser();
          enableButton = (!currentUser || selectedModel.get('userId') !== currentUser.id);
        }
      }

      return enableButton;
    };
  },

  /**
   * @protected
   * Enable 'Delete' when user has 'delete' permission and user is not read only and user is not the current signed on
   * used or the anonymous user.
   */
  bindDeleteButton: function(button) {
    var arr = [
      { observable: this.getUserStore(), events: ['load']},
      { observable: Ext.History, events: ['change']}
    ];

    button.mon(
        AS.Conditions.and(
            AS.Conditions.isPermitted(this.permission + ':delete'),
            AS.Conditions.watchEvents(arr, this.watchEventsHandler(true))
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
   * Deletes a user.
   * @param model user to be deleted
   */
  deleteModel: function(model) {
    var me = this,
        description = me.getDescription(model);

    AS.direct.coreui_User.remove(model.get('userId'), model.get('realm'), function(response) {
      me.getStore('User').load();
      if (Ext.isObject(response) && response.success) {
        AS.Messages.success(AS.I18n.format('Users_Delete_Success', description));
      }
    });
  },

  /**
   * @protected
   * Enable 'ChangePassword' button as appropriate for user's permissions.
   */
  bindSetPasswordButton: function(button) {
    var arr = [
      { observable: this.getUserStore(), events: ['load']},
      { observable: Ext.History, events: ['change']}
    ];

    button.mon(
        AS.Conditions.and(
            AS.Conditions.isPermitted('astrum:userschangepw:create'),
            AS.Conditions.watchEvents(arr, this.watchEventsHandler())
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
   * @override
   * @private
   * Enable 'Change Password' when user has 'astrum:userschangepw:create' permission.
   */
  bindChangePasswordButton: function(button) {
    button.mon(
        AS.Conditions.and(
            AS.Conditions.isPermitted('astrum:userschangepw:create')
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
   */
  changePassword: function(button) {
    var win = button.up('window'),
        password = button.up('form').down('#password').getValue();

    AS.Security.doWithAuthenticationToken(
        AS.I18n.get('Users_ChangePassword_Authenticate_Text'),
        {
          success: function(authToken) {
            AS.direct.coreui_User.changePassword(authToken, win.userId, password, function(response) {
              if (Ext.isObject(response) && response.success) {
                win.close();
                AS.Messages.success(AS.I18n.get('Users_Change_Success'));
              }
            });
          }
        }
    );
  },

  /**
   * @override
   * @private
   * Get an ID from a model. Override if using a model with a synthetic ID
   */
  getModelId: function(model) {
    return model.get('userId');
  },

  reselect: function () {
    var userSearchBox = this.getUserSearchBox();
    if (this.suppressReselect) {
      return;
    }
    if (userSearchBox && !userSearchBox.hasFocus) {
      this.callParent();
    }
  },

  onNavigate: function() {
    var bookmark = AS.Bookmarks.getBookmark();
    var token = bookmark && bookmark.getToken ? bookmark.getToken() : null;
    if (token && token.includes('security/users')) {
      this.reselect();
    }
  },

});
