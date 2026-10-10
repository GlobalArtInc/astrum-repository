
/*global Ext, AS*/

/**
 * I18n helper.
 *
 * @since 3.0
 */
Ext.define('AS.I18n', {
  singleton: true,
  mixins: {
    logAware: 'AS.LogAware'
  },

  /**
   * @private
   * @property {Object}
   */
  keys: {},

  /**
   * @private
   * @property {Object}
   */
  bundles: {},

  /**
   * @private
   * @property {Object}
   */
  translations: null,

  /**
   * Registers English strings, replaced by their translations when the page is shown in another language.
   *
   * @public
   * @param {Object} strings
   */
  register: function (strings) {
    var me = this,
        translations = me.getTranslations();

    Ext.Object.each(strings.keys || {}, function (key, text) {
      me.keys[key] = me.translated(translations[key], text);
    });
    Ext.Object.each(strings.bundles || {}, function (bundle, resources) {
      var merged = {};
      Ext.Object.each(resources, function (key, text) {
        merged[key] = me.translated(translations[bundle + '/' + key], text);
      });
      me.bundles[bundle] = merged;
    });
  },

  /**
   * The translations of the ExtJS dictionaries ({@code extjs-*}) handed over by the page as {@code window.AS_I18N}:
   * plain keys, and bundle keys written as {@code <bundle>/<key>}.
   *
   * @private
   * @returns {Object}
   */
  getTranslations: function () {
    var me = this,
        i18n = window.AS_I18N;

    if (me.translations === null) {
      me.translations = {};
      if (i18n && i18n.dictionaries) {
        Ext.Object.each(i18n.dictionaries, function (name, entries) {
          if (name.indexOf('extjs-') === 0) {
            Ext.apply(me.translations, entries);
          }
        });
      }
    }
    return me.translations;
  },

  /**
   * @private
   */
  translated: function (translation, text) {
    return Ext.isString(translation) && translation !== '' ? translation : text;
  },

  /**
   * Resolves a string from a key.
   *
   * If the key begins with '@' then the remainder is assumed to be a reference to another key and will be resolved.
   *
   * @public
   * @param {String} key
   */
  get: function (key) {
    var text = this.keys[key];
    if (text === null || text === undefined) {
      this.logWarn('Missing I18n key:', key);
      return 'MISSING_I18N:' + key;
    }
    // resolve references
    if (text.charAt(0) === '@') {
      return this.get(text.substring(1, text.length));
    }
    else {
      return text;
    }
  },

  /**
   * @public
   * @param {String} key
   * @param {Object...} values
   * @returns {String}
   */
  format: function (key) {
    var text = this.get(key);
    if (text) {
      var params = Array.prototype.slice.call(arguments);
      // replace first element with text
      params.shift();
      params.unshift(text);
      text = Ext.String.format.apply(this, params);
    }
    return text;
  },

  /**
   * Render a bundle string.
   *
   * @param {Object/String} bundle
   * @param {String} key
   * @param {Object...} [params]
   * @returns {String}
   */
  render: function (bundle, key) {
    var resources, text, params;

    // resolve bundle
    if (Ext.isObject(bundle)) {
      bundle = Ext.getClassName(bundle);
    }

    //<if debug>
    this.logTrace('Resolving bundle:', bundle, 'key:', key);
    //</if>

    resources = this.bundles[bundle];
    if (resources === undefined) {
      this.logWarn('Missing I18n bundle:', bundle);
      return 'MISSING_I18N:' + bundle + ':' + key;
    }

    // resolve text
    text = resources[key];
    if (text === undefined) {
      // handle bundle extension
      var extend = resources['$extend'];
      if (extend !== undefined) {
        params = Array.prototype.slice.call(arguments, 1);
        params.unshift(extend);
        return this.render.apply(this, params);
      }

      this.logWarn('Missing I18n bundle key:', bundle, ':', key);
      return 'MISSING_I18N:' + bundle + ':' + key;
    }

    // resolve references
    if (text.charAt(0) === '@') {
      if (text.indexOf(':') !== -1) {
        // bundle ref <bundle>:[key]
        var items = text.substring(1, text.length).split(':', 2);

        // default to given key if ref missing key
        if (items[1] === '') {
          items[1] = key;
        }

        params = Array.prototype.slice.call(arguments, 2);
        params.unshift(items[1]);
        params.unshift(items[0]);
        return this.render.apply(this, params);
      }
      else {
        // key ref <key>
        text = this.get(text.substring(1, text.length));
      }
    }

    // optionally format with parameters
    if (arguments.length > 2) {
      params = Array.prototype.slice.call(arguments, 2);
      params.unshift(text);
      text = Ext.String.format.apply(this, params);
    }

    return text;
  }
});
