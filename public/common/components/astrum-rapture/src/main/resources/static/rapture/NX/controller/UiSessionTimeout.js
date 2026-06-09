

/**
 * UI Session Timeout controller.
 *
 * @since 3.0
 */
Ext.define('NX.controller.UiSessionTimeout', {
  extend: 'NX.app.Controller',
  requires: ['Ext.ux.ActivityMonitor', 'NX.Messages', 'NX.Security', 'NX.State', 'NX.I18n', 'NX.util.Window'],

  views: [
    'ExpireSession'
  ],

  refs: [
    {
      ref: 'expireSessionWindow',
      selector: 'nx-expire-session'
    }
  ],

  SECONDS_TO_EXPIRE: 30,
  SESSION_ACTIVITY_KEY: 'nx.ui.lastActivityAt',
  TAB_ID_KEY: 'nx.ui.tabId',
  SESSION_EXPIRE_WINDOW_OWNER: 'nx.ui.expireWindowOwner',
  SESSION_EXPIRE_WINDOW_TS: 'nx.ui.expireWindowTs',

  activityMonitor: undefined,

  expirationTicker: undefined,

  activityTrackingInitialized: false,

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      controller: {
        '#State': {
          userchanged: me.setupTimeout,
          uisettingschanged: me.onUiSettingsChanged,
          receivingchanged: me.setupTimeout
        }
      },
      component: {
        'nx-expire-session': {
          beforerender: NX.util.Window.closeWindows,
          afterrender: me.startTicking,
          destroy: function () {
            me.clearExpireWindowOwnership();
          }
        },
        'nx-expire-session button[action=cancel]': {
          click: function (btn) {
            me.clearExpireWindowOwnership();
            me.touchSharedActivity();
            btn.up('window').close();
            me.setupTimeout();
          },
        },
      },
    });
  },

  /**
   * @override
   */
  onLaunch: function () {
    this.setupTimeout();
  },

  /**
   * Initialize activity tracking across tabs.
   * Uses managed listeners (mon) which are automatically cleaned up when controller is destroyed.
   * @private
   */
  initializeActivityTracking: function () {
    var me = this;

    if (me.activityTrackingInitialized) {
      return;
    }
    me.activityTrackingInitialized = true;

    // Initialize shared activity timestamp if not present
    if (!localStorage.getItem(me.SESSION_ACTIVITY_KEY)) {
      me.touchSharedActivity();
    }

    // Set up managed activity listeners to update shared activity on user interactions
    // mon() automatically cleans up listeners when controller is destroyed
    me.mon(Ext.getDoc(), {
      mousedown: { fn: me.touchSharedActivity, scope: me },
      wheel: { fn: me.touchSharedActivity, scope: me, buffer: 250 },
      keydown: { fn: me.touchSharedActivity, scope: me, buffer: 250 },
    });

    // Use native visibilitychange listener for better reliability across browsers
    me.mon(Ext.getDoc(), {
      visibilitychange: function () {
        if (!document.hidden) {
          var uiSettings = NX.State.getValue('uiSettings') || {};
          var sessionTimeout = uiSettings.sessionTimeout;
    
          if (sessionTimeout > 0 && this.isGloballyInactiveForLogout(sessionTimeout)) {
            NX.Security.signOut();
            return;
          }
    
          // treat “coming back” as activity (optional)
          this.touchSharedActivity();
        }
      },
      scope: me
    });
    
  },

  /**
   * Get or create a unique tab ID for this browser tab.
   * @private
   * @returns {String} Tab ID
   */
  getTabId: function () {
    var id = sessionStorage.getItem(this.TAB_ID_KEY);
    if (!id) {
      id = Ext.id(null, 'nx-tab-');
      sessionStorage.setItem(this.TAB_ID_KEY, id);
    }
    return id;
  },

  /**
   * Update the shared last activity timestamp in localStorage.
   * This is called whenever user activity is detected in any tab.
   * @private
   */
  touchSharedActivity: function () {
    localStorage.setItem(this.SESSION_ACTIVITY_KEY, String(Date.now()));
  },

  /**
   * Get the last shared activity timestamp across all tabs.
   * @private
   * @returns {Number} Timestamp in milliseconds, or 0 if not set
   */
  getSharedLastActivity: function () {
    var v = parseInt(localStorage.getItem(this.SESSION_ACTIVITY_KEY), 10);
    return isNaN(v) ? 0 : v;
  },

  /**
   * Check if all tabs are globally inactive for warning (no activity in any tab within timeout window).
   * @private
   * @param {Number} sessionTimeoutMinutes Session timeout in minutes
   * @returns {Boolean} True if globally inactive for warning
   */
  isGloballyInactiveForWarning: function (sessionTimeoutMinutes) {
    var warningMs = this.SECONDS_TO_EXPIRE * 1000;
    var timeoutMs = sessionTimeoutMinutes * 60 * 1000;
    var thresholdMs = Math.max(0, timeoutMs - warningMs);
    return Date.now() - this.getSharedLastActivity() >= thresholdMs;
  },

  /**
   * Check if all tabs are globally inactive for logout (no activity in any tab within timeout window).
   * @private
   * @returns {Boolean} True if globally inactive for logout
   */
  isGloballyInactiveForLogout: function (sessionTimeoutMinutes) {
    var timeoutMs = sessionTimeoutMinutes * 60 * 1000;
    return Date.now() - this.getSharedLastActivity() >= timeoutMs;
  },

  /**
   * Try to acquire ownership of showing the expiration window to prevent multiple tabs from showing it simultaneously.
   * @private
   * @returns {Boolean} True if this tab acquired or already owns showing the expiration window
   */
  tryAcquireExpireWindowOwnership: function () {
    var now = Date.now();
    var owner = localStorage.getItem(this.SESSION_EXPIRE_WINDOW_OWNER);
    var ts = parseInt(localStorage.getItem(this.SESSION_EXPIRE_WINDOW_TS), 10);
    var tabId = this.getTabId();

    // If no owner exists, or ownership expired (older than 60 seconds), acquire it
    if (!owner || isNaN(ts) || now - ts > 60000) {
      localStorage.setItem(this.SESSION_EXPIRE_WINDOW_OWNER, tabId);
      localStorage.setItem(this.SESSION_EXPIRE_WINDOW_TS, String(now));
      return true;
    }

    // If this tab already owns the window, allow it
    return owner === tabId;
  },

  /**
   * Reset UI session timeout when uiSettings.sessionTimeout changes.
   *
   * @private
   * @param {Object} uiSettings
   * @param {Number} uiSettings.sessionTimeout
   * @param {Object} oldUiSettings
   * @param {Number} oldUiSettings.sessionTimeout
   */
  onUiSettingsChanged: function (uiSettings, oldUiSettings) {
    uiSettings = uiSettings || {};
    oldUiSettings = oldUiSettings || {};

    if (uiSettings.sessionTimeout !== oldUiSettings.sessionTimeout) {
      this.setupTimeout();
    }

    if (uiSettings.requestTimeout) {
      this.setRequestTimeout(uiSettings.requestTimeout);
    }
  },

  /**
   * Clear the expire window ownership timestamp.
   * @private Called when the expiration modal is cancelled or destroyed to clear expire window ownership.
   */
  clearExpireWindowOwnership: function () {
    localStorage.removeItem(this.SESSION_EXPIRE_WINDOW_OWNER);
    localStorage.removeItem(this.SESSION_EXPIRE_WINDOW_TS);
  },

  /**
   * @private
   */
  setupTimeout: function () {
    var me = this,
        hasUser = !Ext.isEmpty(NX.State.getUser()),
        uiSettings = NX.State.getValue('uiSettings') || {},
        sessionTimeout = uiSettings['sessionTimeout'],
        requestTimeout = uiSettings['requestTimeout'];

    me.cancelTimeout();
    if ((hasUser && NX.State.isReceiving()) && sessionTimeout > 0) {
      //<if debug>
      me.logDebug('Session expiration enabled for', sessionTimeout, 'minutes');
      //</if>

      // Initialize activity tracking if not already done
      me.initializeActivityTracking();

      me.activityMonitor = Ext.create('Ext.ux.ActivityMonitor', {
        // check every second
        interval: 1000,
        maxInactive: Math.max(0, (sessionTimeout * 60 - me.SECONDS_TO_EXPIRE) * 1000),
        // Gate the idle trigger: only show expiration window if globally inactive
        isInactive: Ext.bind(function () {
          if (!me.isGloballyInactiveForWarning(sessionTimeout)) {
            return;
          }
          me.showExpirationWindow();
        }, me),
      });
      me.activityMonitor.start();
    }

    me.setRequestTimeout(requestTimeout);
  },

  /**
   * @private
   */
  setRequestTimeout: function (timeoutSeconds) {
    if (isNaN(timeoutSeconds)) {
      return;
    }

    var timeoutMilliseconds = timeoutSeconds * 1000;
    Ext.Ajax.setTimeout(timeoutMilliseconds);
    Ext.override(Ext.form.Panel, { timeout: timeoutSeconds });
    Ext.override(Ext.data.Connection, { timeout: timeoutSeconds });
  },

  /**
   * @private
   */
  cancelTimeout: function () {
    var me = this,
        expireSessionView = me.getExpireSessionWindow();

    // close the window if the session has not yet expired or if the server is disconnected
    if (expireSessionView && (!expireSessionView.sessionExpired() || !NX.State.isReceiving())) {
      expireSessionView.close();
    }

    if (me.activityMonitor) {
      me.activityMonitor.stop();
      delete me.activityMonitor;

      //<if debug>
      me.logDebug('Activity monitor disabled');
      //</if>
    }

    if (me.expirationTicker) {
      me.expirationTicker.destroy();
      delete me.expirationTicker;

      //<if debug>
      me.logDebug('Session expiration disabled');
      //</if>
    }
  },

  /**
   * @private
   */
  showExpirationWindow: function () {
    var existing = this.getExpireSessionWindow();
    if (existing) {
      return;
    }

    // Only show expiration window if this tab owns it (prevents multiple tabs from showing it)
    if (this.tryAcquireExpireWindowOwnership()) {
      NX.Messages.warning(NX.I18n.get('UiSessionTimeout_Expire_Message'));
      this.getExpireSessionView().create();
    }
  },

  /**
   * @private
   */
  startTicking: function (win) {
    var me = this;
    var uiSettings = NX.State.getValue('uiSettings') || {};
    var sessionTimeout = uiSettings['sessionTimeout'];

    me.expirationTicker = Ext.util.TaskManager.newTask({
      run: function (count) {
        win.down('label').setText(NX.I18n.format('UiSessionTimeout_Expire_Text', me.SECONDS_TO_EXPIRE - count));
        if (count === me.SECONDS_TO_EXPIRE) {
          // Abort logout if activity resumed in another tab during countdown
          if (!me.isGloballyInactiveForLogout(sessionTimeout)) {
            me.setupTimeout();
            return;
          }

          win.down('label').setText(NX.I18n.get('SignedOut_Text'));
          win.down('button[action=close]').show();
          win.down('button[action=signin]').show();
          win.down('button[action=cancel]').hide();
          NX.Messages.warning(NX.I18n.format('UiSessionTimeout_Expired_Message', sessionTimeout));
          NX.Security.signOut();
        }
      },
      interval: 1000,
      repeat: me.SECONDS_TO_EXPIRE
    });
    me.expirationTicker.start();
  }

});
