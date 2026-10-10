
/*global Ext, AS*/

/**
 * User controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.User', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.util.Base64',
    'AS.Messages',
    'AS.State',
    'AS.I18n',
    'AS.view.header.Mode',
    'AS.util.Window',
    'Ext.Deferred',
    'Ext.Array'
  ],

  views: [
    'header.SignIn',
    'header.SignOut',
    'Authenticate',
    'ExpireSession'
  ],

  refs: [
    {
      ref: 'signInButton',
      selector: 'as-header-signin'
    },
    {
      ref: 'signOutButton',
      selector: 'as-header-signout'
    },
    {
      ref: 'userMode',
      selector: 'as-header-mode[name=user]'
    },
    {
      ref: 'authenticate',
      selector: 'as-authenticate'
    }
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.getApplication().getIconController().addIcons({
      'authenticate': {
        file: 'lock.png',
        variants: ['x16', 'x32']
      }
    });

    me.listen({
      controller: {
        '#State': {
          userchanged: me.onUserChanged
        }
      },
      component: {
        'as-header-panel': {
          afterrender: me.manageButtons
        },
        'as-header-signout': {
          click: me.onClickSignOut
        },
        'as-authenticate button[action=authenticate]': {
          click: me.doAuthenticateAction
        }
      }
    });
  },

  /**
   * @private
   */
  onUserChanged: function (user, oldUser) {
    var me = this;

    if (user && !oldUser) {
      AS.Messages.info(AS.I18n.format('User_SignedIn_Message', user.id));
      me.fireEvent('signin', user);
      if (typeof window.initializeTelemetry == 'function') {
        window.initializeTelemetry();
      }
    }
    else if (!user && oldUser) {
      AS.Messages.info(AS.I18n.get('User_SignedOut_Message'));
      me.fireEvent('signout');
      if (typeof window.initializeTelemetry == 'function') {
        window.initializeTelemetry();
      }
    }

    if (!user) {
      AS.util.Window.closeWindows();
    }

    me.manageButtons();
  },

  /**
   * Returns true if there is an authenticated user.
   *
   * @public
   * @return {boolean}
   */
  hasUser: function () {
    return Ext.isDefined(AS.State.getUser());
  },

  /**
   * Begins the process of retrieving an authentication token that will be used for a subsequent action. The auth token
   * is requested thru the {@code authTokenRequest} event; if no listeners provide a token the user will be prompted
   * for their credentials in order to obtain one.
   *
   * The {@code authTokenRequest} event is sent with an array argument and string argument:
   *    - array: listeners are expected to push a {@code Ext.Deferred} that is resolved with a token or null once
   *        completed. If a listener fails via {@code Ext.Deferred.reject} this will be treated as cancelling the
   *        action.
   *    - string: the message that would be shown in the authentication window
   *
   * @public
   * @param {String} [message] Message to be shown in authentication window
   * @param {Object} [options] TODO
   */
  doWithAuthenticationToken: function (message, options) {
    var me = this,
        token = null,
        handlers = [];

    me.fireEvent('authTokenRequest', handlers, message);

    Ext.Deferred.all(handlers).then(function(tokens) {
      if (Ext.isArray(tokens)) {
        // take the first defined token
        token = Ext.Array.findBy(tokens, function(item) {
          return !!item;
        });
      }
    }, function() {
      token = 'cancel';
    }).always(function() {
      if (token !== 'cancel') {
        if (!token) {
          options = Ext.apply(options || {}, {authenticateAction: me.retrieveAuthenticationToken});

          me.showAuthenticateWindow(message, options);
        }
        else {
          if (Ext.isFunction(options.success)) {
            options.success.call(options.scope, token, options);
          }
        }
      }
      else {
        if (Ext.isFunction(options.failure)) {
          options.failure.call(options.failure, options);
        }
      }
    });
  },

  /**
   * @private
   */
  doAuthenticateAction: function (button) {
    var win = button.up('window');

    // invoke optional authenticateAction callback registered on window
    if (win.options && Ext.isFunction(win.options.authenticateAction)) {
      win.options.authenticateAction.call(this, button);
    }
  },

  /**
   * Shows authenticate window.
   *
   * @private
   * @param {String} [message] Message to be shown in authentication window
   * @param {Object} [options] TODO
   * @param {Object} [user] Optional user object that represents the current user
   */
  showAuthenticateWindow: function (message, options, user) {
    var me = this,
        // AS.State.getUser() is undefined for anonymous subjects; guard so anonymous-with-admin
        // sessions do not crash the re-auth dialog with a NPE (ASTRUM-47114).
        currentUser = AS.State.getUser(),
        username = user ? user.id : (currentUser ? currentUser.id : null),
        win;

    if (!me.getAuthenticate()) {
      win = me.getAuthenticateView().create({message: message, options: options});
      if (username) {
        win.down('form').getForm().setValues({username: username});
        win.down('#password').focus();
      }
    } else {
      if (Ext.isFunction(options.failure)) {
        options.failure.call(options.failure, options);
      }
    }
  },

  /**
   * @private
   */
  retrieveAuthenticationToken: function (button) {
    var win = button.up('window'),
        form = button.up('form'),
        user = AS.State.getUser(),
        values = Ext.applyIf(form.getValues(), {username: user ? user.id : undefined}),
        b64username = AS.util.Base64.encode(values.username),
        b64password = AS.util.Base64.encode(values.password);

    win.getEl().mask(AS.I18n.get('User_Retrieving_Mask'));

    //<if debug>
    this.logDebug('Retrieving authentication token...');
    //</if>

    AS.direct.rapture_Security.authenticationToken(b64username, b64password, function (response) {
      win.getEl().unmask();
      if (Ext.isObject(response) && response.success) {
        win.close();

        // invoke optional success callback registered on window
        if (win.options && Ext.isFunction(win.options.success)) {
          win.options.success.call(win.options.scope, response.data, win.options);
        }
      }
    });
  },

  /**
   * @private
   */
  onClickSignOut: function () {
    var me = this;

    if (me.fireEvent('beforesignout')) {
      me.signOut();
    }
  },

  /**
   * @public
   */
  signOut: function () {
    var me = this;

    me.logDebug('Sign-out');

    // TODO: Mask?

    Ext.Ajax.request({
      url: AS.util.Url.urlOf('service/rapture/session'),
      method: 'DELETE',
      scope: me,
      suppressStatus: true,
      success: function (response) {
        var location = response.getResponseHeader('Location');
        if (location) {
          // No need to clear user state, href navigation clears all state automatically.
          window.location.href = location;
        } else {
          AS.State.setUser(undefined);
        }
      }
    });
  },

  manageButtons: function () {
    var me = this,
        user = AS.State.getUser(),
        signInButton = me.getSignInButton(),
        signOutButton = me.getSignOutButton(),
        userMode = me.getUserMode();

    if (signInButton) {
      if (user) {
        signInButton.hide();
        userMode.show();
        userMode.getViewModel().set('text', user.id);
        userMode.getViewModel().set('tooltip', AS.I18n.format('User_Tooltip', user.id));
        signOutButton.show();
      }
      else {
        signInButton.show();
        userMode.hide();
        signOutButton.hide();
      }
    }
  }

});
