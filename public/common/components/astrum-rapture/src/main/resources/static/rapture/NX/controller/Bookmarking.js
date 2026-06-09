
/*global Ext, NX*/

/**
 * Bookmarking controller.
 *
 * @since 3.0
 */
Ext.define('NX.controller.Bookmarking', {
  extend: 'NX.app.Controller',
  requires: [
    'Ext.History',
    'NX.Bookmark',
    'NX.Bookmarks'
  ],

  /**
   * If this controller had been launched. Becomes true after onLaunch() method is called by ExtJS.
   */
  launched: false,

  /**
   * @override
   */
  init: function () {
    var me = this;

    // The only requirement for this to work is that you must have a hidden field and
    // an iframe available in the page with ids corresponding to Ext.History.fieldId
    // and Ext.History.iframeId.  See history.html for an example.
    Ext.History.useTopWindow = false;
    Ext.History.init();

    me.bindToHistory();
  },

  /**
   * @public
   * @returns {NX.Bookmark} current bookmark
   */
  getBookmark: function () {
    // changing this to getHash rather than getToken. getToken is just a wrapper around getHash that caches the values
    // of getHash, and it is not staying up to date with route changes driven by UI Router.
    return NX.Bookmarks.fromToken(Ext.History.bookmark || Ext.History.getHash());
  },

  /**
   * Sets bookmark to a specified value.
   *
   * @public
   * @param {NX.Bookmark} bookmark new bookmark
   * @param {Object} [caller] whom is asking to bookmark
   */
  bookmark: function (bookmark, caller) {
    var me = this,
        oldValue = me.getBookmark().getToken();

    if (!me.launched) {
      return;
    }

    if (bookmark && oldValue !== bookmark.getToken()) {
      //<if debug>
      me.logDebug('Bookmark:', bookmark.getToken(), (caller ? '(' + caller.self.getName() + ')' : ''));
      //</if>

      Ext.History.bookmark = bookmark.getToken();
      Ext.History.add(bookmark.getToken());
    }
  },

  /**
   * Sets bookmark to a specified value and navigates to it.
   *
   * @public
   * @param {NX.Bookmark} bookmark to navigate to
   * @param {Object} [caller] whom is asking to navigate
   */
  navigateTo: function (bookmark, caller) {
    var me = this;

    if (!me.launched) {
      return;
    }

    if (bookmark) {
      //<if debug>
      me.logDebug('Navigate to:', bookmark.getToken(), (caller ? '(' + caller.self.getName() + ')' : ''));
      //</if>

      me.bookmark(bookmark, caller);
      me.fireEvent('navigate', bookmark);
    }
  },

  /**
   * Navigate to current bookmark.
   *
   * @override
   */
  onLaunch: function () {
    this.launched = true;
    this.navigateTo(this.getBookmark(), this);
  },

  /**
   * Sets bookmark to a specified value and navigates to it.
   *
   * @private
   * @param {String} token to navigate to
   */
  onNavigate: function (token) {
    var me = this;

    if (token !== Ext.History.bookmark) {
      delete Ext.History.bookmark;
      me.navigateTo(NX.Bookmarks.fromToken(token), me);
    }
  },

  /**
   * Start listening to **{@link Ext.History}** change events.
   *
   * @private
   */
  bindToHistory: function () {
    var me = this;

    Ext.History.on('change', me.onNavigate, me);
  }

});
