
/*global Ext, AS*/

/**
 * Helpers to interact with **{@link AS.controller.Bookmarking}** controller.
 *
 * @since 3.0
 */
Ext.define('AS.Bookmarks', {
  singleton: true,
  requires: [
    'AS.Bookmark'
  ],

  /**
   * @private
   * @returns {AS.controller.Bookmarking}
   */
  controller: function () {
    return AS.getApplication().getBookmarkingController();
  },

  /**
   * @see AS.controller.Bookmarking#getBookmark
   */
  getBookmark: function () {
    return this.controller().getBookmark();
  },

  /**
   * @see AS.controller.Bookmarking#bookmark
   */
  bookmark: function (bookmark, caller) {
    return this.controller().bookmark(bookmark, caller);
  },

  /**
   * @see AS.controller.Bookmarking#navigateTo
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
    return this.controller().navigateTo(AS.Bookmarks.fromSegments(bookmark.getSegments().slice(0, -segments)), caller);
  },

  /**
   * Creates a new bookmark.
   *
   * @public
   * @param {String} token bookmark token
   * @returns {AS.Bookmark} created bookmark
   */
  fromToken: function (token) {
    return Ext.create('AS.Bookmark', { token: token });
  },

  /**
   * Creates a new bookmark from provided segments.
   *
   * @public
   * @param {String[]} segments bookmark segments
   * @returns {AS.Bookmark} created bookmark
   */
  fromSegments: function (segments) {
    var token;
    if (Ext.isDefined(segments)) {
      token = Ext.Array.from(segments).join(':');
    }
    return Ext.create('AS.Bookmark', { token: token });
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
