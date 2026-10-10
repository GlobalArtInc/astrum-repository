
/*global Ext, AS*/

/**
 * Features registration controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.Features', {
  extend: 'AS.app.Controller',

  models: [
    'Feature'
  ],
  stores: [
    'Feature',
    'FeatureMenu'
  ],

  statics: {
    /**
     * Always returns true.
     *
     * @returns {boolean}
     */
    alwaysVisible: function () {
      return true;
    },

    /**
     * Always returns false.
     *
     * @returns {boolean}
     */
    alwaysHidden: function () {
      return false;
    }
  },

  /**
   * Registers features.
   *
   * @param {Array/Object} features to be registered
   * @param {Ext.util.Observable} [owner] to be watched to automatically unregister the features if owner is destroyed
   */
  registerFeature: function (features, owner) {
    var me = this;

    if (features) {
      if (owner) {
        owner.on('destroy', Ext.pass(me.unregisterFeature, [features], me), me);
      }
      Ext.each(Ext.Array.from(features), function (feature) {
        var clonedFeature = Ext.clone(feature),
            path;

        if (!clonedFeature.path) {
          throw Ext.Error.raise('Feature missing path');
        }

        if (!clonedFeature.mode) {
          clonedFeature.mode = 'admin';
        }

        if (!clonedFeature.view && clonedFeature.group === true) {
          clonedFeature.view = 'AS.view.feature.Group';
        }

        // complain if there is no view configuration
        if (!clonedFeature.view) {
          me.logError('Missing view configuration for feature at path:', clonedFeature.path);
        }

        path = clonedFeature.path;
        if (path.charAt(0) === '/') {
          path = path.substr(1, path.length);
        }

        me.configureIcon(path, clonedFeature);

        path = clonedFeature.mode + '/' + path;
        clonedFeature.path = '/' + path;

        // auto-set bookmark
        if (!clonedFeature.bookmark) {
          clonedFeature.bookmark = AS.Bookmarks.encode(path).toLowerCase();
        }

        if (Ext.isDefined(clonedFeature.visible)) {
          if (!Ext.isFunction(clonedFeature.visible)) {
            if (clonedFeature.visible) {
              clonedFeature.visible = AS.controller.Features.alwaysVisible;
            }
            else {
              clonedFeature.visible = AS.controller.Features.alwaysHidden;
            }
          }
        }
        else {
          clonedFeature.visible = AS.controller.Features.alwaysVisible;
        }

        me.getStore('Feature').addSorted(me.getFeatureModel().create(clonedFeature));
      });
    }
  },

  /**
   * Un-registers features.
   *
   * @param {Object[]/Object} features to be unregistered
   */
  unregisterFeature: function (features) {
    var me = this;

    if (features) {
      Ext.each(Ext.Array.from(features), function (feature) {
        var clonedFeature = Ext.clone(feature),
            path, model;

        if (!clonedFeature.mode) {
          clonedFeature.mode = 'admin';
        }
        path = clonedFeature.path;
        if (path.charAt(0) === '/') {
          path = path.substr(1, path.length);
        }
        path = clonedFeature.mode + '/' + path;
        clonedFeature.path = '/' + path;

        model = me.getStore('Feature').getById(clonedFeature.id);
        if (model) {
          me.getStore('Feature').remove(model);
        }
      });
    }
  },

  /**
   * @private
   * @param feature
   */
  configureIcon: function (path, feature) {
    var defaultIconName = 'feature-' + feature.mode + '-' + path.toLowerCase().replace(/\//g, '-').replace(/\s/g, '');

    // inline icon registration for feature
    if (feature.iconConfig) {
      var icon = feature.iconConfig;
      delete feature.iconConfig;
      if (icon.name) {
        feature.iconName = icon.name;
      }
      else {
        icon.name = defaultIconName;
      }
      this.getApplication().getIconController().addIcon(icon);
    }

    // default icon name if not set
    if (!feature.iconName) {
      feature.iconName = defaultIconName;
    }
  }

});
