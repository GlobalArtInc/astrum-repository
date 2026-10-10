
/*global Ext, AS*/

/**
 * URL related utils.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.util.RepositoryUrls', {
  singleton: true,
  requires: [
    'AS.util.Url',
    'AS.Assert'
  ],

  mixins: {
    logAware: 'AS.LogAware'
  },

  /**
   * Strategies for building urls to download assets.
   *
   * @private
   */
  repositoryUrlStrategies: {
    maven2: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    apt: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    cocoapods: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    conan: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    conda: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    pub: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name'),
          downloadPath,
          displayName;

      // Pub storage paths and their browse/display equivalents:
      // 1. Package tarballs: /packages/{name}/{version}/{name}-{version}.tar.gz
      //    -> Display: {name}/{version}/{name}-{version}.tar.gz
      //    -> Download: /api/archives/{name}-{version}.tar.gz
      // 2. Package metadata: /api/packages/{name}
      //    -> Display: {name}
      //    -> Download: /api/packages/{name}
      // 3. Version metadata: /api/packages/{name}/versions/{version}
      //    -> Display: {name}/{version}
      //    -> Download: /api/packages/{name}/versions/{version}

      if (assetName.startsWith('/packages/') && assetName.endsWith('.tar.gz')) {
        // For tarballs stored at /packages/{name}/{version}/{filename}
        // Display as: {name}/{version}/{filename}
        var pathWithoutPrefix = assetName.substring('/packages/'.length);
        var parts = pathWithoutPrefix.split('/');
        var filename = parts[parts.length - 1];
        downloadPath = '/api/archives/' + filename;
        displayName = pathWithoutPrefix;
      } else if (assetName.startsWith('/api/packages/') && assetName.indexOf('/versions/') > 0) {
        // For version metadata: /api/packages/{name}/versions/{version}
        // Display as: {name}/{version}
        var versionPath = assetName.substring('/api/packages/'.length);
        var versionParts = versionPath.split('/versions/');
        displayName = versionParts[0] + '/' + versionParts[1];
        downloadPath = assetName;
      } else if (assetName.startsWith('/api/packages/')) {
        // For package metadata: /api/packages/{name}
        // Display as: {name}
        displayName = assetName.substring('/api/packages/'.length);
        downloadPath = assetName;
      } else {
        // Fallback for any other assets
        downloadPath = encodePath(assetName);
        displayName = assetName;
      }

      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + downloadPath, displayName);
    },
    npm: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    nuget: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    r: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    raw: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    rubygems: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    docker: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    pypi: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    yum: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    gitlfs: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    go: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    cargo: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    composer: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    helm: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    p2: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    huggingface: function (me, assetModel) {
      var repositoryName = assetModel.get('repositoryName'),
          assetName = assetModel.get('name');
      return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
    },
    terraform: function (me, assetModel) {
        var repositoryName = assetModel.get('repositoryName'),
            assetName = assetModel.get('name');
          return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
      },
      swift: function (me, assetModel) {
          var repositoryName = assetModel.get('repositoryName'),
              assetName = assetModel.get('name');
          return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
      },
      alpine: function (me, assetModel) {
        var repositoryName = assetModel.get('repositoryName'),
            assetName = assetModel.get('name');
        return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
      },
      ansiblegalaxy: function (me, assetModel) {
          var repositoryName = assetModel.get('repositoryName'),
              assetName = assetModel.get('name');
          return AS.util.Url.asLink(AS.util.Url.relativePath + '/repository/' + encodeURIComponent(repositoryName) + encodePath(assetName), assetName);
      }
  },

  /**
   * Add a strategy to build repository download links for a particular strategy.
   *
   * @public
   */
  addRepositoryUrlStrategy: function (format, strategy) {
    this.repositoryUrlStrategies[format] = strategy;
  },

  /**
   * Creates a link to an asset in a repository.
   *
   * @public
   * @param {Object} assetModel the asset to create a link for
   * @param {String} format the format of the repository storing this asset
   */
  asRepositoryLink: function (assetModel, format) {
    //<if assert>
    AS.Assert.assert(assetModel, 'Expected an assetModel with format: ' + format);
    //</if>
    //<if debug>
    this.logTrace('Creating link for format and asset:', format, assetModel.get('name'));
    //</if>

    var linkStrategy = this.repositoryUrlStrategies[format];

    if (!linkStrategy) {
      // Fallback to a default strategy
      linkStrategy = this.repositoryUrlStrategies.raw;
    }

    return linkStrategy(this, assetModel);
  }
});

/**
 * Breaks down the component path into parts, encodes each part and adds back the slashes, returns the full component path with / not encoded
 *
 * @param uri: full component path
 * @returns component path encoded correctly without slashes included
 */
function encodePath(uri) {
  var parts = uri.split("/");
  for (var i = 0; i < parts.length; i++) {
    parts[i] = encodeURIComponent(parts[i]);
  }
  return parts.join("/");
}
