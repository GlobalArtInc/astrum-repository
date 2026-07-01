
/*global Ext, NX*/

/**
 * CoreUi plugin configuration.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.app.PluginConfig', {
  '@aggregate_priority': 100,

  namespaces: [
    'NX.coreui'
  ],

  requires: [
    'NX.coreui.app.PluginStrings'
  ],

  controllers: [
    {
      id: 'NX.coreui.controller.Assets',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.BrowseableFormats',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Capabilities',
      active: function() {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.PreviewUiSettings',
      active: function() {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.ComponentAssetTree',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin') &&
            NX.State.getValue('browseableformats').length > 0;
      }
    },
    'NX.coreui.controller.FeatureGroups',
    {
      id: 'NX.coreui.controller.HealthCheckInfo',
      active: function () {
        return NX.app.Application.bundleActive('astrum-clm-oss-plugin')
            || NX.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.HealthCheckRepositoryColumn',
      active: function () {
        return NX.app.Application.bundleActive('astrum-clm-oss-plugin')
            || NX.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.HealthCheckRepositoryConfiguration',
      active: function () {
        return NX.app.Application.bundleActive('astrum-clm-oss-plugin')
            || NX.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Clm',
      active: function () {
        return NX.app.Application.bundleActive('astrum-clm-oss-plugin')
            || NX.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.FirewallRepositoryColumn',
      active: function () {
        return NX.app.Application.bundleActive('astrum-clm-oss-plugin')
            || NX.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.LdapServers',
      active: function () {
        return NX.app.Application.bundleActive('astrum-ldap-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Outreach',
      active: function () {
        return NX.app.Application.bundleActive('astrum-outreach-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Repositories',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Nodes',
      active: function () {
            return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Privileges',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Search',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.SearchApt',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-apt');
      }
    },
    {
      id: 'NX.coreui.controller.SearchCargo',
      active: function() {
        return NX.app.Application.bundleActive('astrum-repository-cargo');
      }
    },
    {
      id: 'NX.coreui.controller.SearchComposer',
      active: function() {
        return NX.app.Application.bundleActive('astrum-repository-composer');
      }
    },
    {
      id: 'NX.coreui.controller.SearchConan',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-conan');
      }
    },
    {
      id: 'NX.coreui.controller.SearchConda',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-conda');
      }
    },
    {
      id: 'NX.coreui.controller.SearchPub',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-pub');
      }
    },
    {
      id: 'NX.coreui.controller.SearchGolang',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-golang');
      }
    },
    {
      id: 'NX.coreui.controller.SearchCocoapods',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-cocoapods');
      }
    },
    {
      id: 'NX.coreui.controller.SearchDocker',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-docker');
      }
    },
    {
      id: 'NX.coreui.controller.SearchGitLfs',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-gitlfs');
      }
    },
    {
      id: 'NX.coreui.controller.SearchHelm',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-helm');
      }
    },
    {
      id: 'NX.coreui.controller.SearchHuggingFace',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-huggingface');
      }
    },
    {
      id: 'NX.coreui.controller.SearchMaven',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-maven');
      }
    },
    {
      id: 'NX.coreui.controller.SearchNpm',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-npm');
      }
    },
    {
      id: 'NX.coreui.controller.SearchNuget',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-nuget');
      }
    },
    {
      id: 'NX.coreui.controller.SearchPyPi',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-pypi');
      }
    },
    {
      id: 'NX.coreui.controller.SearchR',
      active: function() {
        return NX.app.Application.bundleActive('astrum-repository-r');
      }
    },
    {
      id: 'NX.coreui.controller.SearchRaw',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-raw');
      }
    },
    {
      id: 'NX.coreui.controller.SearchRubygems',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-rubygems');
      }
    },
    {
      id: 'NX.coreui.controller.SearchYum',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-yum');
      }
    },
    {
      id: 'NX.coreui.controller.SearchP2',
      active: function () {
        return NX.app.Application.bundleActive('astrum-repository-p2');
      }
    },
    {
      id: 'NX.coreui.controller.SslCertificates',
      active: function () {
        return NX.app.Application.bundleActive('astrum-ssl-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.SslTrustStore',
      active: function () {
        return NX.app.Application.bundleActive('astrum-ssl-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Tasks',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.UploadComponent',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.Users',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.HealthCheckWarnings',
      active: function () {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.react.ReactViewController',
      active: function() {
        return NX.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'NX.coreui.controller.SearchAnsiblegalaxy',
      active: function() {
        return NX.app.Application.bundleActive('astrum-repository-ansible-galaxy');
      }
    },
    {
          id: 'NX.coreui.controller.SearchTerraform',
          active: function() {
              return NX.app.Application.bundleActive('astrum-repository-terraform');
          }
      },
    {
      id: 'NX.coreui.controller.SearchSwift',
      active: function() {
        return NX.app.Application.bundleActive('astrum-repository-swift');
      }
    }
  ]
});
