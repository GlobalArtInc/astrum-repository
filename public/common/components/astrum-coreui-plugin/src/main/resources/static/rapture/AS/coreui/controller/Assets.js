
/*global Ext, AS*/

/**
 * Assets controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.Assets', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.Bookmarks',
    'AS.coreui.util.Maven2ComponentActionHandler',
    'AS.Dialogs',
    'AS.I18n',
    'Ext.util.Format'
  ],

  mixins: {
    componentUtils: 'AS.coreui.mixin.ComponentUtils'
  },

  views: [
    'component.AssetContainer',
    'component.AssetInfo',
    'component.AssetAttributes',
    'component.AssetList',
    'component.ComponentDetails'
  ],

  refs: [
    {ref: 'assetContainer', selector: 'as-coreui-component-assetcontainer'},
    {ref: 'assetList', selector: 'grid[assetContainerSource=true]'},
    {ref: 'assetInfo', selector: 'as-coreui-component-assetinfo'},
    {ref: 'deleteAssetButton', selector: 'as-coreui-component-assetcontainer button[action=deleteAsset]'},
    {ref: 'componentAssetInfo', selector: 'as-coreui-component-componentassetinfo'},
    {ref: 'viewVulnerabilitiesButton', selector: 'as-coreui-component-componentassetinfo button[action=viewVulnerabilities]'},
    {ref: 'componentList', selector: 'grid[componentList=true]'},
    {ref: 'componentDetails', selector: 'as-coreui-component-details'},
    {ref: 'deleteComponentButton', selector: 'as-coreui-component-details button[action=deleteComponent]'},
    {ref: 'browseComponentButton', selector: 'as-coreui-component-details button[action=browseComponent]'},
    {ref: 'rootContainer', selector: 'as-main'}
  ],

  DEPENDENCY_SNIPPET_PANEL_ID: 'snippetPanel',

  /**
   * @override
   */
  init: function () {
    var me = this;

    var fileExtensionIcons = [{},
          '3gp', '7z', 'ace', 'ai', 'aif', 'aiff', 'amr', 'asf', 'asx', 'bat', 'bin', 'bmp', 'bup', 'cab', 'cbr', 'cda',
          'cdl', 'cdr', 'chm', 'dat', 'divx', 'dll', 'dmg', 'doc', 'dss', 'dvf', 'dwg', 'eml', 'eps', 'exe', 'fla',
          'flv', 'gif', 'gz', 'hqx', 'htm', 'html', 'ifo', 'indd', 'iso', 'jar', 'jpeg', 'jpg', 'lnk', 'log', 'm4a',
          'm4b', 'm4p', 'm4v', 'mcd', 'mdb', 'mid', 'mov', 'mp2', 'mp4', 'mpeg', 'mpg', 'msi', 'mswmm', 'ogg', 'pdf',
          'png', 'pps', 'ps', 'psd', 'pst', 'ptb', 'pub', 'qbb', 'qbw', 'qxd', 'ram', 'rar', 'rm', 'rmvb', 'rtf', 'sea',
          'ses', 'sit', 'sitx', 'ss', 'swf', 'tgz', 'thm', 'tif', 'tmp', 'torrent', 'ttf', 'txt', 'vcd', 'vob', 'wav',
          'wma', 'wmv', 'wps', 'xls', 'xpi', 'zip'
    ].reduce(function(icons, extension) {
      icons['asset-type-' + extension] = {
        file: 'file_extension_' + extension + '.png',
        variants: ['x16', 'x32']
      };
      return icons;
    });

    ['debian', 'json'].forEach(function(fileName) {
      fileExtensionIcons['asset-type-' + fileName] = {
        file: fileName + '.png',
        variants: ['x16', 'x32']
      };
    });

    var formatIcons = [{}, 'code', 'ruby'].reduce(function(icons, extension) {
      icons['asset-type-' + extension] = {
        file: 'page_white_' + extension + '.png',
        variants: ['x16', 'x32']
      };
      return icons;
    });

    me.getApplication().getIconController().addIcons(fileExtensionIcons);
    me.getApplication().getIconController().addIcons(formatIcons);
    me.getApplication().getIconController().addIcons({
      'asset-type-default': {
        file: 'page_white.png',
        variants: ['x16', 'x32']
      }
    });

    me.listen({
      component: {
        'as-coreui-component-assetcontainer': {
          updated: me.showAssetInfo.bind(me)
        },
        'as-coreui-component-details': {
          updated: me.showComponentDetails
        },
        'as-coreui-component-asset-list': {
          updated: me.loadAssets
        },
        'as-coreui-component-assetcontainer button[action=deleteAsset]': {
          click: me.deleteAsset
        },
        'as-coreui-component-componentassetinfo button[action=viewVulnerabilities]': {
          click: me.mixins.componentUtils.viewVulnerabilities
        },
        'as-coreui-component-details button[action=deleteComponent]': {
          click: me.deleteComponent
        },
        'as-coreui-component-details button[action=browseComponent]': {
          click: me.browseComponent
        }
      }
    });

    me.repositoryStore = Ext.create('AS.coreui.store.RepositoryReference', {remote: true, autoLoad: true});
  },

  /**
   * Shows information about selected component/asset.
   *
   * @private
   * @param {AS.coreui.view.component.AssetContainer} container asset container
   * @param {AS.coreui.model.Asset} assetModel selected asset
   */
  showAssetInfo: function (container, assetModel) {
    var info = container.down('as-coreui-component-assetinfo'),
        attributes = container.down('as-coreui-component-assetattributes'),
        componentDetails = this.getComponentDetails(),
        componentModel = componentDetails && componentDetails.componentModel,
        format, dependencySnippets;

    if (!info) {
      container.addTab(
          {
            xtype: 'as-coreui-component-assetinfo',
            title: AS.I18n.get('Component_AssetInfo_Info_Title'),
            itemId: 'assetInfo',
            weight: 10
          }
      );
      info = container.down('as-coreui-component-assetinfo');
    }
    info.setAssetModel(assetModel);

    if (!attributes) {
      container.addTab(
          {
            xtype: 'panel',
            ui: 'as-inset',
            title: AS.I18n.get('Component_AssetInfo_Attributes_Title'),
            itemId: 'attributeInfo',
            weight: 30,
            autoScroll: true,
            items: [
              {xtype: 'as-coreui-component-assetattributes'}
            ]
          }
      );

      attributes = container.down('as-coreui-component-assetattributes');
    }
    attributes.setAssetModel(assetModel);

    if (componentModel) {
      format = componentModel.get('format');
      dependencySnippets = AS.getApplication().getDependencySnippetController()
          .getDependencySnippets(format, componentModel, assetModel);

      if (dependencySnippets && dependencySnippets.length > 0) {
        this.getDependencySnippetPanel(container).setDependencySnippets(format, dependencySnippets);
        container.showTab(this.DEPENDENCY_SNIPPET_PANEL_ID);
      } else {
        container.hideTab(this.DEPENDENCY_SNIPPET_PANEL_ID);
      }
    }
  },

  getDependencySnippetPanel: function(container) {
    if (!container.down('as-info-dependency-snippet-panel')) {
      container.addTab(
          {
            xtype: 'as-info-dependency-snippet-panel',
            title: AS.I18n.get('DependencySnippetPanel_Title'),
            collapsible: false,
            itemId: this.DEPENDENCY_SNIPPET_PANEL_ID,
            weight: 20
          }
      );
    }

    return container.down('as-info-dependency-snippet-panel');
  },

  showComponentDetails: function (container, componentModel) {
    var repositoryInfo = {},
        componentInfo = {};

    if (componentModel) {
      repositoryInfo[AS.I18n.get('Search_Assets_Repository')] = componentModel.get('repositoryName');
      repositoryInfo[AS.I18n.get('Search_Assets_Format')] = componentModel.get('format');
      componentInfo[AS.I18n.get('Search_Assets_Group')] = componentModel.get('group');
      componentInfo[AS.I18n.get('Search_Assets_Name')] = componentModel.get('name');
      componentInfo[AS.I18n.get('Search_Assets_Version')] = componentModel.get('version');

      container.down('#repositoryInfo').showInfo(repositoryInfo);
      container.down('#componentInfo').showInfo(componentInfo);
    }

    this.updateDeleteButtonVisibility();
    this.updateBrowseButtonVisibility();
  },

  /**
   * @private
   *
   * Filter asset store based on component model.
   *
   * @param {AS.coreui.view.component.AssetList} grid assets grid
   * @param {AS.coreui.model.Component} componentModel component owning the assets to be loaded
   */
  loadAssets: function (grid, componentModel) {
    var assetStore = grid.getStore();

    if (componentModel) {
      assetStore.clearFilter(true);
      assetStore.addFilter([
        {
          property: 'repositoryName',
          value: componentModel.get('repositoryName')
        },
        {
          property: 'componentModel',
          value: JSON.stringify(componentModel.getData())
        }
      ]);
    }
  },

  /**
   * Update asset shown in asset container.
   *
   * @public
   */
  updateAssetContainer: function (gridView, td, cellIndex, assetModel) {
    var me = this;

    me.getAssetContainer().refreshInfo(assetModel);

    me.getRepository(assetModel.get('repositoryName'), function (repository) {
      me.updateDeleteAssetButton(repository, assetModel);
    });
  },

  /**
   * Get the current repository
   *
   * @param repositoryName name of repository
   * @param callback function which will get passed the repository
   *
   * @private
   */
  getRepository: function(repositoryName, callback) {
    var repositoryStore = this.repositoryStore,
        repository = repositoryStore.getAt(repositoryStore.find('name', repositoryName));

    if (repository) {
      callback(repository);
    }
    else {
      repositoryStore.load(function() {
        callback(repositoryStore.getAt(repositoryStore.find('name', repositoryName)));
      });
    }
  },

  /**
   * Remove selected component.
   *
   * @private
   */
  deleteComponent: function() {
    var me = this,
        componentDetails = me.getComponentDetails(),
        componentModel, componentId, componentVersion;

    if (componentDetails) {
      componentModel = componentDetails.componentModel;
      componentVersion = componentModel.get('version');
      componentId = componentModel.get('name') + '/' + componentVersion;
      AS.Dialogs.askConfirmation(AS.I18n.get('ComponentDetails_Delete_Title'), Ext.htmlEncode(componentId), function() {
        AS.direct.coreui_Component.deleteComponent(JSON.stringify(componentModel.getData()), function(response) {
          if (Ext.isObject(response) && response.success) {
            me.refreshComponentList();
            AS.Bookmarks.navigateBackSegments(AS.Bookmarks.getBookmark(), 1);
            AS.Messages.success(AS.I18n.format('ComponentDetails_Delete_Success', componentId));
          }
        });
      });
    }
  },

  /**
   * Enable 'Delete' when user has 'delete' permission. Button will be hidden for group repositories.
   *
   * @private
   */
  updateDeleteButtonVisibility: function() {
    var me = this,
        componentModel = me.fetchComponentModelFromView(),
        formatSpecificActionHandler = me.getFormatSpecificActionHandler(componentModel),
        button = me.getDeleteComponentButton();

    if (componentModel &&
        (!formatSpecificActionHandler ||
            !formatSpecificActionHandler.updateDeleteButtonVisibility(button, componentModel))
    ) {
      me.getRepository(componentModel.get('repositoryName'), function(repository) {
        me.updateDeleteComponentButton(repository, componentModel);
      });
    }
  },

  /**
   * Enable 'Browse' button when format supports it.
   *
   * @private
   */
  updateBrowseButtonVisibility: function() {
    var componentModel = this.fetchComponentModelFromView(),
        formatSpecificActionHandler = this.getFormatSpecificActionHandler(componentModel),
        button = this.getBrowseComponentButton();

    if (!formatSpecificActionHandler || !formatSpecificActionHandler.updateBrowseButtonVisibility(button, componentModel)) {
      button.hide();
    }
  },

  /**
   * Get the format specific component details actions
   */
  getFormatSpecificActionHandler: function(componentModel) {
    var format = componentModel && componentModel.get('format'),
        alias = format && 'as-coreui-' + format.toLowerCase() + '-component-action-handler';

    return alias && Ext.ClassManager.getByAlias(alias);
  },

  browseComponent: function() {
    var componentModel = this.fetchComponentModelFromView(),
        assetModel = this.fetchAssetModelFromView(),
        formatSpecificActionHandler = this.getFormatSpecificActionHandler(componentModel);

    if (formatSpecificActionHandler && formatSpecificActionHandler.browseComponent) {
      formatSpecificActionHandler.browseComponent(componentModel, assetModel);
    }
  },

  /**
   * @private
   * Remove selected asset.
   */
  deleteAsset: function () {
    var me = this,
        assetList = me.getAssetList(),
        assetInfo = me.getAssetInfo();

    if (assetInfo) {
      var asset = assetInfo.assetModel;
      AS.Dialogs.askConfirmation(AS.I18n.get('AssetInfo_Delete_Title'), Ext.htmlEncode(asset.get('name')), function () {
        AS.direct.coreui_Component.deleteAsset(asset.getId(), asset.get('repositoryName'), function (response) {
          if (Ext.isObject(response) && response.success) {
            assetList.getSelectionModel().deselectAll();
            //Manually managing sync'ing the local AssetStore as AssetStore.load() won't run a callback if the load
            //results in no data being returned.
            var assetStore = assetList.getStore();

            response.data.forEach(function(assetName) {
              assetStore.remove(assetStore.findRecord('name', assetName));
            });

            me.navigateBackOnAssetDelete(asset.get('componentId'), assetStore);
            AS.Messages.success(AS.I18n.format('AssetInfo_Delete_Success', asset.get('name')));
          }
        });
      });
    }
  },

  /**
   * Decide whether or not we should navigate back to a parent Component or to the prior page.
   * @private
   * @param {String} componentId
   * @param {Ext.data.Store} assetStore 
   */
  navigateBackOnAssetDelete: function(componentId, assetStore) {
    var me = this;
    if (!me.getComponentDetails() || !componentId || me.countAssetsInComponent(assetStore, componentId) > 0) {
      // Asset being deleted either does not have an associated component in scope, or is not the last Asset
      //<if debug>
      me.logDebug('Asset deleted with no component in scope or as last remaining asset');
      //</if>
      Ext.util.History.back();
    }
    else {
      //<if debug>
      me.logDebug('Asset deleted with component in scope');
      //</if>
      me.refreshComponentList();
      AS.Bookmarks.navigateBackSegments(AS.Bookmarks.getBookmark(), 2);
    }
  },

  countAssetsInComponent: function(assetStore, componentId) {
    var count = 0;
    assetStore.each(function(asset){
      if (asset.get('componentId') === componentId) {
        count++;
      }
    });

    return count;
  },

  fetchComponentModelFromView: function() {
    return this.getComponentDetails().componentModel;
  },

  fetchAssetModelFromView: function() {
    var assetList = this.getAssetList(),
        assetStore = assetList && assetList.getStore();
    return assetStore && assetStore.getAt(0);
  },

  /**
   * @private
   * Refresh component list.
   */
  refreshComponentList: function () {
    var componentList = this.getComponentList();

    componentList.getSelectionModel().deselectAll();
    // delay refresh of component list because in case of search results it takes a while till removal is
    // propagated to elastic search results. Not 100% but better then still showing
    setTimeout(function() {
      componentList.getStore().load();
    }, 1000);
  }

});
