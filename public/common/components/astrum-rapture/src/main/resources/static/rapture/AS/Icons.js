
/*global Ext, AS*/

/**
 * Helpers to interact with Icon controller.
 *
 * @since 3.0
 */
Ext.define('AS.Icons', {
  singleton: true,
  requires: [
    'Ext.DomHelper',
    'AS.util.Url'
  ],
  mixins: {
    logAware: 'AS.LogAware'
  },

  /**
   * Helper to get the CSS class for a named icon with optional variant.
   *
   * @public
   */
  cls: function (name, variant) {
    // translate . -> _ to avoid problems with CSS selector syntax muck
    var cls = 'as-icon-' + name.replace('.', '_');
    if (variant) {
      cls += '-' + variant;
    }
    return cls;
  },

  /**
   * Helper to get html text for a named icon with variant.
   *
   * @public
   */
  img: function(name, variant) {
    return Ext.DomHelper.markup({
      tag: 'img',
      src: Ext.BLANK_IMAGE_URL,
      cls: this.cls(name, variant),
      alt: name,
      'aria-hidden': true
    });
  },

  /**
   * Helper to get a cache-busted URL for an icon name + variant + optional extension.
   *
   * @param name
   * @param [variant]
   * @param [ext] The file extension to use, png if not set.
   * @returns {string}
   */
  url: function(name, variant, ext) {
    var file = name;

    if (ext === undefined) {
      ext = 'png';
    }
    file += '.' + ext;

    return this.url2(file, variant);
  },

  /**
   * Helper to get a cache-busted URL for an icon file + optional variant.
   *
   * @param file
   * @param [variant]
   * @returns {string}
   */
  url2: function(file, variant) {
    var url = AS.util.Url.relativePath + '/static/rapture/resources/icons/';
    if (variant) {
      url += variant + '/';
    }
    url += file;
    return AS.util.Url.cacheBustingUrl(url);
  }

});
