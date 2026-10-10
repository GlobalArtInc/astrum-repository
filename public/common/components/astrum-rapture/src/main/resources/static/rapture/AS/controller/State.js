
/*global Ext, AS*/

/**
 * State controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.State', {
  extend: 'AS.app.Controller',
  requires: [
    'Ext.direct.Manager',
    'AS.Dialogs',
    'AS.Messages',
    'AS.I18n',
    'Ext.Ajax'
  ],

  models: [
    'State'
  ],
  stores: [
    'State'
  ],

  /**
   * @private
   */
  disconnectedTimes: 0,

  /**
   * Max number of times to show a warning, before disabling the UI.
   *
   * @private
   */
  maxDisconnectWarnings: 3,

  /**
   * True when state is received from server.
   *
   * @private
   */
  receiving: false,

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      controller: {
        '#State': {
          userchanged: me.onUserChanged,
          uisettingschanged: me.onUiSettingsChanged,
          licensechanged: me.onLicenseChanged,
          serveridchanged: me.reloadWhenServerIdChanged,
          clusteridchanged: me.reloadWhenServerIdChanged
        }
      },
      store: {
        '#State': {
          add: me.onEntryAdded,
          update: me.onEntryUpdated,
          remove: me.onEntryRemoved
        }
      }
    });
  },

  /**
   * Install initial state, primed from app.js
   *
   * @override
   */
  onLaunch: function () {
    var me = this;

    //<if debug>
    me.logTrace('Initial state:', AS.app.state);
    //</if>

    var uiSettings = AS.app.state['uiSettings'];

    AS.State.setBrowserSupported(
        !Ext.isIE || (Ext.isIE9p && Ext.isIE11m)
    );
    AS.State.setValue('debug', AS.app.debug);
    AS.State.setValue('receiving', false);

    // set uiSettings by the end so it does not start state pulling till all initial state hashes are known
    // this avoids unnecessary sending of state from server
    delete AS.app.state['uiSettings'];
    AS.State.setValues(AS.app.state);
    AS.State.setValues({ uiSettings: uiSettings });

    //<if debug>
    me.logInfo('State primed');
    //</if>
  },

  /**
   * @public
   * @returns {Boolean} true when status is being received from server
   */
  isReceiving: function () {
    return this.receiving;
  },

  getValue: function(key, defaultValue) {
    var model = this.getStore('State').getById(key),
        value;

    if (model) {
      value = model.get('value');
      if (Ext.isDefined(value)) {
        return value;
      }
    }
    return defaultValue;
  },

  /**
   * @public
   * @param {String} key
   * @param {Object} value
   * @param {String} [hash]
   */
  setValue: function (key, value, hash) {
    var me = this,
        store = me.getStore('State'),
        model = store.getById(key),
        hasValue = Ext.isDefined(value) && value !== null;

    if (!hasValue && model) {
      store.remove(model);
    }
    else if (hasValue && !model) {
      store.add(me.getStateModel().create({ key: key, value: value, hash: hash }));
    }
    else if (hash && !Ext.Object.equals(hash, model.get('hash'))) {
      model.set('hash', hash);

      if (key === 'user' && model.get('value').id === value.id) {
        model.set('value', value, { silent: true });
        me.fireEvent('userAuthenticated', key, value);
      }

      if (Ext.typeOf(value) !== 'object' && value !== model.get('value')) {
        model.set('value', value);
      }

      if (!Ext.Object.equals(value, model.get('value'))) {
        model.set('value', value);
      }
    }
    else if (!hash && hasValue) {
      model.set('hash', hash);
      model.set('value', value);
    }

    store.commitChanges();

    if (me.statusProvider) {
      if (hasValue && hash) {
        me.statusProvider.baseParams[key] = hash;
      }
      else {
        delete me.statusProvider.baseParams[key];
      }
    }

  },

  setValues: function (map) {
    var me = this,
        hash, valueToSet;

    if (map) {
      Ext.Object.each(map, function (key, value) {
        valueToSet = value;
        if (Ext.isObject(value) && Ext.isDefined(value.hash) && Ext.isDefined(value.value)) {
          hash = value.hash;
          valueToSet = value.value;
        }
        if (Ext.isDefined(valueToSet)) {
          if (!Ext.isPrimitive(valueToSet) && !Ext.isArray(valueToSet)
              && Ext.ClassManager.getByAlias('as.state.' + key)) {
            valueToSet = Ext.ClassManager.instantiateByAlias('as.state.' + key, valueToSet);
          }
        }
        me.setValue(key, valueToSet, hash);
      });
    }
  },

  onEntryAdded: function (store, models) {
    var me = this;
    Ext.each(models, function (model) {
      me.notifyChange(model.get('key'), model.get('value'));
    });
  },

  onEntryUpdated: function (store, model, operation, modifiedFieldNames) {
    if ((operation === Ext.data.Model.EDIT) && modifiedFieldNames.indexOf('value') > -1) {
      this.notifyChange(model.get('key'), model.get('value'), model.modified.value);
    }
  },

  onEntryRemoved: function (store, models) {
    models.forEach(function(model) {
      this.notifyChange(model.get('key'), undefined, model.get('value'));
    }, this);
  },

  notifyChange: function (key, value, oldValue) {
    var me = this;

    //<if debug>
    me.logTrace('Changed:', key, '->', (value ? value : '(deleted)'));
    //</if>

    me.fireEvent(key.toLowerCase() + 'changed', value, oldValue);
    me.fireEvent('changed', key, value, oldValue);
  },

  /**
   * Reset state pooling when uiSettings.statusInterval changes.
   *
   * @private
   */
  onUiSettingsChanged: function (uiSettings, oldUiSettings) {
    var me = this,
        newStatusInterval, newStatusIntervalMs, oldStatusIntervalMs;

    uiSettings = uiSettings || {};
    oldUiSettings = oldUiSettings || {};

    if (uiSettings.debugAllowed !== oldUiSettings.debugAllowed) {
      AS.State.setValue('debug', uiSettings.debugAllowed && (AS.global.location.search === '?debug'));
    }

    if (uiSettings.title !== oldUiSettings.title) {
      AS.global.document.title = AS.global.document.title.replace(oldUiSettings.title, uiSettings.title);
    }

    if (me.statusProvider) {
      oldStatusIntervalMs = me.statusProvider.interval;
    }

    newStatusInterval = uiSettings.statusIntervalAnonymous;
    if (AS.State.getUser()) {
      newStatusInterval = uiSettings.statusIntervalAuthenticated;
    }

    newStatusIntervalMs = newStatusInterval * 1000;

    if (newStatusInterval > 0) {
      if (newStatusIntervalMs !== oldStatusIntervalMs) {
        if (me.statusProvider) {
          me.statusProvider.disconnect();
          me.receiving = false;
        }
        me.statusProvider = Ext.direct.Manager.addProvider({
          type: 'polling',
          url: AS.direct.api.POLLING_URLS.rapture_State_get,
          interval: newStatusIntervalMs,
          baseParams: {
          },
          listeners: {
            data: me.onServerData,
            scope: me
          }
        });

        //<if debug>
        me.logDebug('State pooling configured for', newStatusInterval, 'seconds');
        //</if>

        // fire one request for state manually to not wait for the polling interval
        me.refreshNow();
      }
    }
    else {
      if (me.statusProvider) {
        me.statusProvider.disconnect();
      }

      //<if debug>
      me.logDebug('State pooling disabled');
      //</if>
    }
  },

  /**
   * On sign-in/sign-out update status interval.
   *
   * @private
   */
  onUserChanged: function (user, oldUser) {
    var me = this;
    var uiSettings;

    if (Ext.isDefined(user) !== Ext.isDefined(oldUser)) {
      uiSettings = AS.State.getValue('uiSettings');

      // fire one request for state manually on login if the statusInterval is 0
      if (user && !oldUser && uiSettings) {
        uiSettings.statusIntervalAuthenticated === 0 && me.refreshNow()
      }

      me.onUiSettingsChanged(uiSettings, uiSettings);
    }
  },

  /**
   * Called when there is new data from state callback.
   *
   * @private
   */
  onServerData: function (provider, event) {
    var me = this;
    if (event.data) {
      me.onSuccess(event);
    }
    else {
      me.onError(event);
    }
  },

  /**
   * Called when state pooling was successful.
   *
   * @private
   */
  onSuccess: function (event) {
    var me = this,
        serverId = me.getValue('serverId'),
        clusterId = me.getValue('clusterId'),
        state = event.data.data,
        clustered = Ext.isDefined(state.nodes) && state.nodes.value.enabled,
        oldClusterId = state.clusterId ? state.clusterId.value : clusterId,
        oldServerId = state.serverId ? state.serverId.value : serverId;

    me.receiving = true;

    // re-enable the UI we are now connected again
    if (me.disconnectedTimes > 0) {
      me.disconnectedTimes = 0;
      AS.Messages.success(AS.I18n.get('State_Reconnected_Message'));
    }

    AS.State.setValue('receiving', true);

    if ( (clustered && !me.reloadWhenServerIdChanged(clusterId, oldClusterId)) ||
        (!clustered && !me.reloadWhenServerIdChanged(serverId, oldServerId)) ) {
      me.setValues(state);
    }
    me.trackUser();
    // TODO: Fire global refresh event
  },

  trackUser: function() {
    var userType;
    return function() {
      if (window.aptrinsic) {
        const user = this.getValue('user');
        var newUserType;
        if (!user) {
          newUserType = 'anonymous';
        }
        else if (user.authenticatedRealms) {
          newUserType = user.administrator ? 'administrator' : 'user';
        }
        if (newUserType && userType != newUserType) {
          userType = newUserType;
          window.aptrinsic('set', 'globalContext', { 'usertype': newUserType });
        }
      }
    };
  }(),

  /**
   * Called when state pooling failed.
   *
   * @private
   */
  onError: function (event) {
    var me = this;

    if (event.code === 'xhr') {
      if (event.xhr.status === 402) {
        AS.State.setValue('license', Ext.apply(Ext.clone(AS.State.getValue('license')), { installed: false }));
      }
      else {
        me.receiving = false;

        // we appear to have lost the server connection
        me.disconnectedTimes = me.disconnectedTimes + 1;

        AS.State.setValue('receiving', false);

        if (me.disconnectedTimes <= me.maxDisconnectWarnings) {
          AS.Messages.warning(AS.I18n.get('State_Disconnected_Message'));
        }

        // Give up after a few attempts and disable the UI
        if (me.disconnectedTimes > me.maxDisconnectWarnings) {
          AS.Messages.error(AS.I18n.get('State_Disconnected_Message'));

          // Stop polling
          me.statusProvider.disconnect();

          // FIXME: i18n
          // Show the UI with a modal dialog error
          AS.Dialogs.showError(
              'Server disconnected',
              'There is a problem communicating with the server',
              {
                buttonText: {
                  ok: 'Retry'
                },

                fn: function () {
                  // retry after the dialog is dismissed
                  me.statusProvider.connect();
                }
              }
          );
        }
      }
    }
    else if (event.type === 'exception') {
      AS.Messages.error(event.message);
    }
  },

  /**
   * Refreshes status from server on demand.
   *
   * @public
   */
  refreshNow: function () {
    var me = this;

    // directly query for state
    Ext.Ajax.request({
      url: AS.direct.api.POLLING_URLS.rapture_State_get,
      scope: me,
      success: function(response) {
        var text = response && response.responseText;

        if (text != null) {
          me.onServerData(null, Ext.isObject(text) || Ext.isArray(text) ? text : Ext.decode(text));
        }
      }
    });

    if (me.statusProvider) {
      me.statusProvider.disconnect();
      me.statusProvider.connect();
    }
  },

  /**
   * Show messages about license.
   *
   * @private
   * @param {Object} license
   * @param {Number} license.installed
   * @param {Object} oldLicense
   * @param {Number} oldLicense.installed
   */
  onLicenseChanged: function (license, oldLicense) {
    if (license && oldLicense) {
      if (license.installed && !oldLicense.installed) {
        AS.Messages.success(AS.I18n.get('State_Installed_Message'));
      }
      else if (!license.installed && oldLicense.installed) {
        AS.Messages.warning(AS.I18n.get('State_Uninstalled_Message'));
      }
    }
  },

  reloadWhenServerIdChanged: function (serverId, oldServerId) {
    if (oldServerId && (serverId !== oldServerId) && !Ext.String.startsWith(serverId, 'ignore')) {
      // FIXME: i18n
      AS.Dialogs.showInfo(
          'Server restarted',
          'Application will be reloaded as server has been restarted',
          {
            fn: function () {
              AS.global.location.reload();
            }
          }
      );
      return true;
    }
    return false;
  }

});
