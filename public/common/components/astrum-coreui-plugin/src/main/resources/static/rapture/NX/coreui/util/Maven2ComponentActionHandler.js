
/*global Ext, NX*/

/**
 * Maven2 Component Details Provider.
 *
 * @since 3.14
 */
Ext.define('NX.coreui.util.Maven2ComponentActionHandler', {
  alias: 'nx-coreui-maven2-component-action-handler',
  singleton: true,
  requires: [
      'NX.Bookmarks',
      'NX.I18n'
  ],

  isSnapshot: function(componentModel) {
    var id;

    if (!componentModel) {
      return false;
    }

    id = componentModel.get('id');
    return (componentModel.get('format') === 'maven2') &&
        id.slice(-'-SNAPSHOT'.length) === '-SNAPSHOT';
  },

  updateDeleteButtonVisibility: function(button, componentModel) {
    var visibilityUpdated;

    if (this.isSnapshot(componentModel)) {
      button.hide();
      visibilityUpdated = true;
    } else {
      visibilityUpdated = false;
    }

    return visibilityUpdated;
  },

  updateBrowseButtonVisibility: function(button, componentModel) {
    if (this.isSnapshot(componentModel)) {
      button.show();
    } else {
      button.hide();
    }

    return true;
  },

  /**
   * Browse to the selected component.
   *
   * @private
   */
  browseComponent: function(componentModel, assetModel) {
    var repositoryName, componentGroup, componentName,
        attributes, version, baseVersion, path;

    if (componentModel && assetModel) {
      repositoryName = componentModel.get('repositoryName');
      componentGroup = componentModel.get('group').replace(/\./g, '/');
      componentName = componentModel.get('name');

      attributes = assetModel.get('attributes');
      version = attributes.maven2 && attributes.maven2.version;
      baseVersion = attributes.maven2 && attributes.maven2.baseVersion;

      path = 'browse/browse:' + encodeURIComponent(repositoryName) + ':' +
          encodeURIComponent(componentGroup + '/' + componentName + '/' + baseVersion + '/' + version);

      NX.Bookmarks.navigateTo(NX.Bookmarks.fromToken(path));
    }
  }


});
