
/*global Ext, AS*/

/**
 * Astrum application.
 *
 * @since 3.0
 */
Ext.define('AS.app.Application', {
  extend: 'Ext.app.Application',

  requires: [
    'Ext.Ajax',
    'Ext.Error',
    'Ext.direct.Manager',
    'Ext.state.Manager',
    'Ext.state.LocalStorageProvider',
    'Ext.util.Cookies',
    'Ext.util.LocalStorage',
    'AS.util.Url',
    'AS.I18n',
    'AS.State'
  ],

  mixins: {
    logAware: 'AS.LogAware'
  },

  uses: [
    // container overrides
    'AS.ext.container.Container',

    // direct overrides
    'AS.ext.direct.RemotingProvider',
    'AS.ext.form.action.DirectLoad',
    'AS.ext.form.action.DirectSubmit',

    // form overrides
    'AS.ext.form.FieldContainer',
    'AS.ext.form.field.Base',
    'AS.ext.form.field.Checkbox',
    'AS.ext.form.field.Display',
    'AS.ext.form.field.Number',

    // panel overrides
    'AS.ext.panel.Panel',
    'AS.ext.panel.ResponsivePanel',
    'AS.ext.panel.Header',

    // toolbar overrides
    'AS.ext.toolbar.Toolbar',

    // custom form fields
    'AS.ext.form.OptionalFieldSet',
    'AS.ext.form.field.Email',
    'AS.ext.form.field.Password',
    'AS.ext.form.field.RegExp',
    'AS.ext.form.field.Url',
    'AS.ext.form.field.ClearableComboBox',
    'AS.ext.form.field.DateDisplayField',
    'AS.ext.form.field.ValueSet',
    'AS.ext.SearchBox',
    'Ext.ux.form.ItemSelector',

    // grid plugins
    'AS.ext.grid.plugin.FilterBox',
    'AS.ext.grid.plugin.RemoteFilterBox',
    'AS.ext.grid.plugin.Filtering',

    // grid overrides
    'AS.ext.grid.Panel',
    'AS.ext.grid.column.Column',
    'AS.ext.grid.column.Date',

    // custom grid columns
    'AS.ext.grid.column.Icon',
    'AS.ext.grid.column.CopyLink',

    // view overrides
    'AS.ext.view.BoundList',

    // animated card layout
    'AS.ext.layout.container.Card',

    // Override for ExtJS 6.6
    'AS.ext.chart.legend.SpriteLegend'
  ],

  name: 'AS',

  /**
   * Store application instance in "AS.application".
   */
  appProperty: 'application',

  /**
   * Relative to /index.html
   */
  appFolder: 'static/rapture/AS',

  paths: {
    'Ext.ux': 'static/rapture/Ext/ux',
    'Ext.patch': 'static/rapture/Ext/patch'
  },

  /**
   * Always active controllers.
   */
  controllers: [
    'Copy',
    'DependencySnippet',
    'Logging',
    'State',
    'Bookmarking',
    'ExtDirect',
    'Features',
    'Icon',
    'KeyNav',
    'Permissions'
  ],

  /**
   * Managed controller configurations.
   *
   * @private
   * @property {Ext.util.MixedCollection}
   */
  managedControllers: undefined,

  statics: {
    alwaysActive: function () {
      return true;
    },
    defaultActivation: function () {
      return AS.app.Application.supportedBrowser();
    },
    supportedBrowser: function () {
      return AS.State.isBrowserSupported();
    },
    unsupportedBrowser: function () {
      return !AS.app.Application.supportedBrowser();
    },
    licensed: function () {
      return !AS.State.requiresLicense() || AS.State.isLicenseValid();
    },
    unlicensed: function () {
      return !AS.app.Application.licensed();
    },
    licenseExpired: function() {
      var daysToLicenseExpiry = AS.State.getDaysToLicenseExpiry();
      return AS.app.Application.licensed() && daysToLicenseExpiry ? daysToLicenseExpiry < 0 : false; 
    },
    debugMode: function () {
      return AS.State.getValue('debug') === true;
    },
    bundleActive: function (symbolicName) {
      return AS.State.getValue('activeBundles').findIndex(function (item) { return item.includes(symbolicName) }) > -1;
    },
    capabilityActive: function (typeName) {
      return Ext.Array.contains(AS.State.getValue('capabilityActiveTypes'), typeName);
    },
    capabilityCreated: function (typeName) {
      return Ext.Array.contains(AS.State.getValue('capabilityCreatedTypes'), typeName);
    }
  },

  /**
   * Flag to indicate that app is ready.
   *
   * @public
   * @property {Boolean}
   * @readonly
   */
  ready: false,

  /**
   * @override
   * @param {AS.app.Application} app this class
   */
  init: function (app) {
    var me = this,
        csrfToken = Ext.util.Cookies.get('AS-ANTI-CSRF-TOKEN'),
        basePath, hostname;

    //<if debug>
    me.logInfo('Initializing');
    me.logDebug(me.managedControllers.getCount(), 'managed controllers');
    //</if>

    // Configure blank image URL
    Ext.BLANK_IMAGE_URL = AS.util.Url.relativePath + '/static/rapture/resources/images/s.gif';

    if (!csrfToken) {
      basePath = AS.util.Url.baseUrl.substring(window.location.origin.length) || null;
      hostname = window.location.hostname;
      if ((Ext.isEdge || Ext.isIE) && hostname && hostname.indexOf('.') === -1) {
        // IE & Edge won't set a cookie for domains without a TLD
        hostname = null;
      }
      csrfToken = Math.random().toString();
      Ext.util.Cookies.set('AS-ANTI-CSRF-TOKEN', csrfToken, null, basePath, hostname);
    }

    Ext.Ajax.setDefaultHeaders({
      'X-Astrum-UI': 'true',
      'AS-ANTI-CSRF-TOKEN': csrfToken
    });

    app.initErrorHandler();
    app.initDirect();
    app.initState();
  },
  // },

  /**
   * Hook into browser error handling (in order to log them).
   *
   * @private
   */
  initErrorHandler: function () {
    var me = this,
        originalOnError = AS.global.onerror;

    // FIXME: This needs further refinement, seems like javascript errors are lost in Firefox (but show up fine in Chrome)

    // pass unhandled errors to application error handler
    Ext.Error.handle = function (err) {
      me.handleError(err);
    };

    // FIXME: This will catch more errors, but duplicates messages for ext errors
    // FIXME: Without this however some javascript errors will go unhandled
    AS.global.onerror = function (msg, url, line) {
      me.handleError({ msg: msg + ' (' + url + ':' + line + ')' });

      // maybe delegate to original onerror handler?
      if (originalOnError) {
        originalOnError(msg, url, line);
      }
    };

    //<if debug>
    me.logDebug('Configured error handling');
    //</if>
  },

  /**
   * Handle application error.
   *
   * @private
   */
  handleError: function (error) {
    AS.Messages.error(this.errorAsString(error));
  },

  /**
   * Customize error to-string handling.
   *
   * Ext.Error.toString() assumes instance, but raise(String) makes anonymous object.
   *
   * @private
   */
  errorAsString: function (error) {
    var className = error.sourceClass || '',
        methodName = error.sourceMethod ? '.' + error.sourceMethod + '(): ' : '',
        msg = error.msg || '(No description provided)';
    return className + methodName + msg;
  },

  /**
   * Initialize Ex.Direct remote providers.
   *
   * @private
   */
  initDirect: function () {
    var remotingProvider;

    remotingProvider = Ext.direct.Manager.addProvider(AS.direct.api.REMOTING_API);

    // configure Ext.Direct buffer-window milliseconds
    remotingProvider.enableBuffer = 10;

    // disable retry
    remotingProvider.maxRetries = 0;

    // default request timeout to 60 seconds
    // note this will be overridden by UiSessionTimeout.js where we load the user config
    remotingProvider.timeout = 60 * 1000;

    //<if debug>
    this.logDebug('Configured Ext.Direct');
    //</if>
  },

  /**
   * Initialize state manager.
   *
   * @private
   */
  initState: function () {
    var me = this,
        provider;

    // If local storage is supported install state provider
    if (Ext.util.LocalStorage.supported) {
      provider = Ext.create('Ext.state.LocalStorageProvider');
      Ext.state.Manager.setProvider(provider);
      //<if debug>
      me.logDebug('Configured state provider: local');
      //</if>
    }
    else {
      //<if debug>
      me.logWarn('Local storage not supported; state management not supported');
      //</if>
    }

    //<if debug>
    if (provider) {
      provider.on('statechange', function (provider, key, value, opts) {
        me.logTrace('State changed:', key, '->', (value ? value : '(deleted)'));
      });
    }
    //</if>
  },

  /**
   * Starts the application.
   *
   * @public
   */
  start: function () {
    var me = this, becomeReady;

    //<if debug>
    me.logInfo('Starting');
    //</if>

    me.syncManagedControllers();
    me.listen({
      controller: {
        '#State': {
          changed: me.syncManagedControllers
        }
      }
    });

    becomeReady = function () {
      // Hide the loading mask immediately after ExtJS initializes.
      // Permissions are loaded asynchronously in background (ASTRUM-52583).
      // This enables immediate UI interactivity even for users with 400+ roles.
      Ext.get('loading').remove();
      Ext.fly('loading-mask').animate({opacity: 0, remove: true});

      // mark app as ready
      me.logInfo('Ready');
      me.ready = true;
    };

    // Wait for ExtJS application to be ready (does NOT wait for permissions)
    waitForExtJS(becomeReady);
  },

  /**
   * Fired when synchronizing controllers and changes were detected.
   *
   * @event controllerschanged
   */

  /**
   * Create / Destroy managed controllers based on their active status.
   *
   * @private
   */
  syncManagedControllers: function () {
    var me = this,
        ref, initializedControllers = [],
        changes = false;

    //<if debug>
    me.logDebug('Refreshing controllers');
    //</if>

    // destroy all controllers that are become inactive
    me.managedControllers.eachKey(function (key) {
      ref = me.managedControllers.get(key);
      if (!ref.active()) {
        if (ref.controller) {
          changes = true;

          //<if debug>
          me.logDebug('Destroying controller:', key);
          //</if>

          ref.controller.fireEvent('destroy', ref.controller);

          // private reference to controller.eventbus
          ref.controller.eventbus.unlisten(ref.controller.id);

          if (Ext.isFunction(ref.controller.onDestroy)) {
            ref.controller.onDestroy();
          }
          me.controllers.remove(ref.controller);
          ref.controller.clearManagedListeners();
          if (Ext.isFunction(ref.controller.destroy)) {
            ref.controller.destroy();
          }
          delete ref.controller;
        }
      }
    });

    // create & init all controllers that become active
    me.managedControllers.eachKey(function (key) {
      ref = me.managedControllers.get(key);
      if (ref.active()) {
        if (!ref.controller) {
          changes = true;

          //<if debug>
          me.logDebug('Initializing controller:', key);
          //</if>

          ref.controller = me.getController(key);
          initializedControllers.push(ref.controller);
        }
      }
    });

    // launch any initialized controller
    Ext.each(initializedControllers, function (controller) {
      controller.onLaunch(me);
    });
    // finish init on any initialized controller
    Ext.each(initializedControllers, function (controller) {
      controller.finishInit(me);
    });

    if (changes) {
      // TODO shall we do this on each refresh?
      me.getIconController().installStylesheet();
      me.fireEvent('controllerschanged');
    }
  }
});

/**
 * Poll until Ext.getApplication() returns a truthy value, then invoke callback.
 * NOTE: Does NOT wait for permissions to be loaded (ASTRUM-52583) - the UI
 * should render immediately and permissions load asynchronously.
 * @param callback to perform when the ExtJS application is ready
 */
function waitForExtJS(callback) {
  const interval = setInterval(function () {
    if (Ext.getApplication()) {
      clearInterval(interval);
      callback();
    }
  }, 50);
}
