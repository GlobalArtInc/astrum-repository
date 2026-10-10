/*global Ext, AS*/

/**
 * Usage snippets of the formats without an interface plugin of their own: the commands and manifest lines that fetch a
 * component, or one of its assets, from the repository it is shown in.
 */
Ext.define('AS.coreui.controller.DependencySnippets', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.I18n',
    'AS.util.Url'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this,
        controller = AS.getApplication().getDependencySnippetController();

    Ext.Object.each(me.generators, function(format, generator) {
      controller.addDependencySnippetGenerator(format, function(componentModel, assetModel) {
        var coordinates = me.coordinates(componentModel, assetModel);
        return coordinates.repositoryName ? generator.call(me, coordinates) : [];
      });
    });
  },

  /**
   * The values the generators read: the component coordinates, and the repository and asset path they come from.
   *
   * @private
   */
  coordinates: function(componentModel, assetModel) {
    var value = function(model, field) {
          var result = model && model.get(field);
          return result ? String(result) : '';
        },
        repositoryName = value(assetModel, 'repositoryName') || value(componentModel, 'repositoryName');

    return {
      group: value(componentModel, 'group'),
      name: value(componentModel, 'name'),
      version: value(componentModel, 'version'),
      repositoryName: repositoryName,
      repositoryUrl: AS.util.Url.baseUrl + '/repository/' + encodeURIComponent(repositoryName) + '/',
      registryUrl: value(assetModel, 'registryUrl') || value(componentModel, 'registryUrl'),
      path: value(assetModel, 'name').replace(/^\/+/, '')
    };
  },

  /**
   * @private
   */
  file: function(name) {
    return AS.I18n.format('DependencySnippet_File_Description', name);
  },

  /**
   * @private
   */
  setup: function() {
    return AS.I18n.get('DependencySnippet_Setup_Description');
  },

  /**
   * The URL of an asset path, each segment encoded.
   *
   * @private
   */
  assetUrl: function(coordinates, path) {
    return coordinates.repositoryUrl + path.split('/').map(encodeURIComponent).join('/');
  },

  /**
   * The asset path without its last segments, or an empty string when it has no more segments.
   *
   * @private
   */
  parentPath: function(path, levels) {
    var segments = path.split('/');
    return segments.length > levels ? segments.slice(0, segments.length - levels).join('/') : '';
  },

  /**
   * A name the client configuration can use for the repository.
   *
   * @private
   */
  remoteName: function(coordinates) {
    return coordinates.repositoryName.toLowerCase().replace(/[^a-z0-9_-]+/g, '-');
  },

  generators: {
    alpine: function(c) {
      var repository = this.assetUrl(c, this.parentPath(c.path, 2)).replace(/\/$/, '');
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'apk',
          snippetText: 'apk add --repository ' + repository + ' ' + c.name + '=' + c.version
        }, {
          displayName: '/etc/apk/repositories',
          description: this.file('/etc/apk/repositories'),
          snippetText: repository
        }
      ];
    },

    apt: function(c) {
      var snippets = [];
      if (c.name && c.version) {
        snippets.push({
          displayName: 'apt',
          snippetText: 'sudo apt install ' + c.name + '=' + c.version
        });
      }
      if (/\.deb$/.test(c.path)) {
        snippets.push({
          displayName: 'curl',
          snippetText: 'curl -fLO ' + this.assetUrl(c, c.path)
        });
      }
      return snippets;
    },

    cargo: function(c) {
      var registry = this.remoteName(c);
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'cargo',
          snippetText: 'cargo add ' + c.name + '@' + c.version + ' --registry ' + registry
        }, {
          displayName: '.cargo/config.toml',
          description: this.setup(),
          snippetText: '[registries.' + registry + ']\nindex = "sparse+' + c.repositoryUrl + '"'
        }, {
          displayName: 'Cargo.toml',
          description: this.file('Cargo.toml'),
          snippetText: c.name + ' = { version = "' + c.version + '", registry = "' + registry + '" }'
        }
      ];
    },

    cocoapods: function(c) {
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'Podfile',
          description: this.file('Podfile'),
          snippetText: 'source \'' + c.repositoryUrl + '\'\n\npod \'' + c.name + '\', \'' + c.version + '\''
        }
      ];
    },

    composer: function(c) {
      var name = c.group ? c.group + '/' + c.name : c.name;
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'Composer',
          snippetText: 'composer require ' + name + ':' + c.version
        }, {
          displayName: 'composer config',
          description: this.setup(),
          snippetText: 'composer config repositories.' + this.remoteName(c) + ' composer ' + c.repositoryUrl
        }, {
          displayName: 'composer.json',
          description: this.file('composer.json'),
          snippetText: '"' + name + '": "' + c.version + '"'
        }
      ];
    },

    conan: function(c) {
      var remote = this.remoteName(c),
          reference = c.name + '/' + c.version + (c.group ? '@' + c.group : '');
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'Conan',
          snippetText: 'conan install --requires=' + reference + ' --remote=' + remote
        }, {
          displayName: 'conan remote',
          description: this.setup(),
          snippetText: 'conan remote add ' + remote + ' ' + c.repositoryUrl
        }, {
          displayName: 'conanfile.txt',
          description: this.file('conanfile.txt'),
          snippetText: '[requires]\n' + reference
        }, {
          displayName: 'conanfile.py',
          description: this.file('conanfile.py'),
          snippetText: 'requires = "' + reference + '"'
        }
      ];
    },

    conda: function(c) {
      var subdir = c.group ? c.path.indexOf('/' + c.group + '/') : -1,
          channel = this.assetUrl(c, subdir > 0 ? c.path.substring(0, subdir) : '').replace(/\/$/, '');
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'conda',
          snippetText: 'conda install -c ' + channel + ' ' + c.name + '=' + c.version
        }, {
          displayName: 'environment.yml',
          description: this.file('environment.yml'),
          snippetText: 'channels:\n  - ' + channel + '\ndependencies:\n  - ' + c.name + '=' + c.version
        }
      ];
    },

    docker: function(c) {
      var manifest = /^v2\/(.+)\/manifests\/([^\/]+)$/.exec(c.path),
          name = manifest ? manifest[1] : c.name,
          reference = manifest ? manifest[2] : c.version,
          image, shortName;
      if (!c.registryUrl || !name || !reference) {
        return [];
      }
      image = c.registryUrl + '/' + name + (reference.indexOf(':') > 0 ? '@' : ':') + reference;
      shortName = name.substring(name.lastIndexOf('/') + 1);
      return [
        {
          displayName: 'Docker',
          snippetText: 'docker pull ' + image
        }, {
          displayName: 'docker login',
          description: this.setup(),
          snippetText: 'docker login ' + c.registryUrl.split('/')[0]
        }, {
          displayName: 'Dockerfile',
          snippetText: 'FROM ' + image
        }, {
          displayName: 'Docker Compose',
          snippetText: 'services:\n  ' + shortName + ':\n    image: ' + image
        }, {
          displayName: 'Kubernetes',
          snippetText: 'spec:\n  containers:\n    - name: ' + shortName + '\n      image: ' + image
        }
      ];
    },

    go: function(c) {
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'go get',
          snippetText: 'GOPROXY=' + c.repositoryUrl + ' go get ' + c.name + '@' + c.version
        }, {
          displayName: 'GOPROXY',
          description: this.setup(),
          snippetText: 'go env -w GOPROXY=' + c.repositoryUrl + ',direct'
        }, {
          displayName: 'go.mod',
          description: this.file('go.mod'),
          snippetText: 'require ' + c.name + ' ' + c.version
        }
      ];
    },

    helm: function(c) {
      var repository = this.remoteName(c);
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'Helm',
          snippetText: 'helm install ' + c.name + ' ' + repository + '/' + c.name + ' --version ' + c.version
        }, {
          displayName: 'helm repo add',
          description: this.setup(),
          snippetText: 'helm repo add ' + repository + ' ' + c.repositoryUrl
        }, {
          displayName: 'helm pull',
          snippetText: 'helm pull ' + repository + '/' + c.name + ' --version ' + c.version
        }, {
          displayName: 'Chart.yaml',
          description: this.file('Chart.yaml'),
          snippetText: 'dependencies:\n  - name: ' + c.name + '\n    version: ' + c.version + '\n    repository: ' +
              c.repositoryUrl
        }
      ];
    },

    npm: function(c) {
      var name = (c.group ? '@' + c.group + '/' : '') + c.name;
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'npm',
          snippetText: 'npm install ' + name + '@' + c.version + ' --registry=' + c.repositoryUrl
        }, {
          displayName: 'Yarn',
          snippetText: 'yarn add ' + name + '@' + c.version + ' --registry ' + c.repositoryUrl
        }, {
          displayName: 'pnpm',
          snippetText: 'pnpm add ' + name + '@' + c.version + ' --registry=' + c.repositoryUrl
        }, {
          displayName: '.npmrc',
          description: this.setup(),
          snippetText: (c.group ? '@' + c.group + ':' : '') + 'registry=' + c.repositoryUrl
        }, {
          displayName: 'package.json',
          description: this.file('package.json'),
          snippetText: '"' + name + '": "' + c.version + '"'
        }
      ];
    },

    nuget: function(c) {
      var source = c.repositoryUrl + 'index.json';
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: '.NET CLI',
          snippetText: 'dotnet add package ' + c.name + ' --version ' + c.version + ' --source ' + source
        }, {
          displayName: 'Package Manager',
          snippetText: 'Install-Package ' + c.name + ' -Version ' + c.version + ' -Source ' + source
        }, {
          displayName: 'nuget.config',
          description: this.file('nuget.config'),
          snippetText: '<packageSources>\n  <add key="' + c.repositoryName + '" value="' + source + '" />\n' +
              '</packageSources>'
        }, {
          displayName: 'PackageReference',
          description: this.file('.csproj'),
          snippetText: '<PackageReference Include="' + c.name + '" Version="' + c.version + '" />'
        }, {
          displayName: 'Paket CLI',
          snippetText: 'paket add ' + c.name + ' --version ' + c.version
        }
      ];
    },

    p2: function(c) {
      return [
        {
          displayName: 'Eclipse',
          description: AS.I18n.get('DependencySnippet_UpdateSite_Description'),
          snippetText: c.repositoryUrl
        }
      ];
    },

    pypi: function(c) {
      var index = c.repositoryUrl + 'simple/';
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'pip',
          snippetText: 'pip install --index-url ' + index + ' ' + c.name + '==' + c.version
        }, {
          displayName: 'uv',
          snippetText: 'uv pip install --index-url ' + index + ' ' + c.name + '==' + c.version
        }, {
          displayName: 'pip.conf',
          description: this.setup(),
          snippetText: '[global]\nindex-url = ' + index
        }, {
          displayName: 'Poetry',
          snippetText: 'poetry source add --priority=primary ' + this.remoteName(c) + ' ' + index + '\n' +
              'poetry add ' + c.name + '==' + c.version
        }, {
          displayName: 'requirements.txt',
          description: this.file('requirements.txt'),
          snippetText: c.name + '==' + c.version
        }
      ];
    },

    r: function(c) {
      if (!c.name || !c.version) {
        return [];
      }
      return [
        {
          displayName: 'R',
          snippetText: 'install.packages("' + c.name + '", repos = "' + c.repositoryUrl + '")'
        }, {
          displayName: 'remotes',
          snippetText: 'remotes::install_version("' + c.name + '", version = "' + c.version + '", repos = "' +
              c.repositoryUrl + '")'
        }, {
          displayName: 'DESCRIPTION',
          description: this.file('DESCRIPTION'),
          snippetText: 'Imports: ' + c.name + ' (== ' + c.version + ')'
        }
      ];
    },

    raw: function(c) {
      var url;
      if (!c.path) {
        return [];
      }
      url = this.assetUrl(c, c.path);
      return [
        {
          displayName: 'curl',
          snippetText: 'curl -fLO ' + url
        }, {
          displayName: 'wget',
          snippetText: 'wget ' + url
        }
      ];
    }
  }
});
