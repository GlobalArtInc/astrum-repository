
/*global Ext, AS*/

/**
 * CoreUi plugin configuration.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.app.PluginConfig', {
  '@aggregate_priority': 100,

  namespaces: [
    'AS.coreui'
  ],

  requires: [
    'AS.coreui.app.PluginStrings'
  ],

  controllers: [
    {
      id: 'AS.coreui.controller.Assets',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.BrowseableFormats',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Capabilities',
      active: function() {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.DependencySnippets',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.ComponentAssetTree',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin') &&
            AS.State.getValue('browseableformats').length > 0;
      }
    },
    'AS.coreui.controller.FeatureGroups',
    {
      id: 'AS.coreui.controller.HealthCheckInfo',
      active: function () {
        return AS.app.Application.bundleActive('astrum-clm-oss-plugin')
            || AS.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.HealthCheckRepositoryColumn',
      active: function () {
        return AS.app.Application.bundleActive('astrum-clm-oss-plugin')
            || AS.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.HealthCheckRepositoryConfiguration',
      active: function () {
        return AS.app.Application.bundleActive('astrum-clm-oss-plugin')
            || AS.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Clm',
      active: function () {
        return AS.app.Application.bundleActive('astrum-clm-oss-plugin')
            || AS.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.FirewallRepositoryColumn',
      active: function () {
        return AS.app.Application.bundleActive('astrum-clm-oss-plugin')
            || AS.app.Application.bundleActive('astrum-clm-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.LdapServers',
      active: function () {
        return AS.app.Application.bundleActive('astrum-ldap-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Outreach',
      active: function () {
        return AS.app.Application.bundleActive('astrum-outreach-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Repositories',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Nodes',
      active: function () {
            return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Privileges',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Search',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.SearchApt',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-apt');
      }
    },
    {
      id: 'AS.coreui.controller.SearchCargo',
      active: function() {
        return AS.app.Application.bundleActive('astrum-repository-cargo');
      }
    },
    {
      id: 'AS.coreui.controller.SearchComposer',
      active: function() {
        return AS.app.Application.bundleActive('astrum-repository-composer');
      }
    },
    {
      id: 'AS.coreui.controller.SearchConan',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-conan');
      }
    },
    {
      id: 'AS.coreui.controller.SearchConda',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-conda');
      }
    },
    {
      id: 'AS.coreui.controller.SearchPub',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-pub');
      }
    },
    {
      id: 'AS.coreui.controller.SearchGolang',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-golang');
      }
    },
    {
      id: 'AS.coreui.controller.SearchCocoapods',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-cocoapods');
      }
    },
    {
      id: 'AS.coreui.controller.SearchDocker',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-docker');
      }
    },
    {
      id: 'AS.coreui.controller.SearchGitLfs',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-gitlfs');
      }
    },
    {
      id: 'AS.coreui.controller.SearchHelm',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-helm');
      }
    },
    {
      id: 'AS.coreui.controller.SearchHuggingFace',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-huggingface');
      }
    },
    {
      id: 'AS.coreui.controller.SearchMaven',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-maven');
      }
    },
    {
      id: 'AS.coreui.controller.SearchNpm',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-npm');
      }
    },
    {
      id: 'AS.coreui.controller.SearchNuget',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-nuget');
      }
    },
    {
      id: 'AS.coreui.controller.SearchPyPi',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-pypi');
      }
    },
    {
      id: 'AS.coreui.controller.SearchR',
      active: function() {
        return AS.app.Application.bundleActive('astrum-repository-r');
      }
    },
    {
      id: 'AS.coreui.controller.SearchRaw',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-raw');
      }
    },
    {
      id: 'AS.coreui.controller.SearchRubygems',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-rubygems');
      }
    },
    {
      id: 'AS.coreui.controller.SearchYum',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-yum');
      }
    },
    {
      id: 'AS.coreui.controller.SearchP2',
      active: function () {
        return AS.app.Application.bundleActive('astrum-repository-p2');
      }
    },
    {
      id: 'AS.coreui.controller.SslCertificates',
      active: function () {
        return AS.app.Application.bundleActive('astrum-ssl-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.SslTrustStore',
      active: function () {
        return AS.app.Application.bundleActive('astrum-ssl-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Tasks',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.UploadComponent',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.Users',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.HealthCheckWarnings',
      active: function () {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.react.ReactViewController',
      active: function() {
        return AS.app.Application.bundleActive('astrum-coreui-plugin');
      }
    },
    {
      id: 'AS.coreui.controller.SearchAnsiblegalaxy',
      active: function() {
        return AS.app.Application.bundleActive('astrum-repository-ansible-galaxy');
      }
    },
    {
          id: 'AS.coreui.controller.SearchTerraform',
          active: function() {
              return AS.app.Application.bundleActive('astrum-repository-terraform');
          }
      },
    {
      id: 'AS.coreui.controller.SearchSwift',
      active: function() {
        return AS.app.Application.bundleActive('astrum-repository-swift');
      }
    }
  ]
});
