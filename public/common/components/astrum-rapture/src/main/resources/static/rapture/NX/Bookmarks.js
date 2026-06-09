
/*global Ext, NX*/

/**
 * Helpers to interact with **{@link NX.controller.Bookmarking}** controller.
 *
 * @since 3.0
 */
Ext.define('NX.Bookmarks', {
  singleton: true,
  requires: [
    'NX.Bookmark'
  ],

  /**
   * @private
   * @returns {NX.controller.Bookmarking}
   */
  controller: function () {
    return NX.getApplication().getBookmarkingController();
  },

  /**
   * @see NX.controller.Bookmarking#getBookmark
   */
  getBookmark: function () {
    return this.controller().getBookmark();
  },

  /**
   * @see NX.controller.Bookmarking#bookmark
   */
  bookmark: function (bookmark, caller) {
    return this.controller().bookmark(bookmark, caller);
  },

  /**
   * @see NX.controller.Bookmarking#navigateTo
   */
  navigateTo: function (bookmark, caller) {
    return this.controller().navigateTo(bookmark, caller);
  },

  /**
   * Navigate back by removing one or more segments from the given bookmark.
   * @param bookmark
   * @param segments
   * @param caller
   * @returns {*}
   */
  navigateBackSegments: function(bookmark, segments, caller) {
    return this.controller().navigateTo(NX.Bookmarks.fromSegments(bookmark.getSegments().slice(0, -segments)), caller);
  },

  /**
   * Creates a new bookmark.
   *
   * @public
   * @param {String} token bookmark token
   * @returns {NX.Bookmark} created bookmark
   */
  fromToken: function (token) {
    return Ext.create('NX.Bookmark', { token: token });
  },

  /**
   * Creates a new bookmark from provided segments.
   *
   * @public
   * @param {String[]} segments bookmark segments
   * @returns {NX.Bookmark} created bookmark
   */
  fromSegments: function (segments) {
    var token;
    if (Ext.isDefined(segments)) {
      token = Ext.Array.from(segments).join(':');
    }
    return Ext.create('NX.Bookmark', { token: token });
  },

  /**
   * Encodes the value suitable to be used as a bookmark token.
   * (eliminate spaces and lower case)
   *
   * @param value to be encoded
   * @returns {String} encoded value
   */
  encode: function (value) {
    if (!Ext.isString(value)) {
      throw Ext.Error.raise('Value to be encoded must be a String');
    }
    return value.replace(/\s/g, '');
  }

});
