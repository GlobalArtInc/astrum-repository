
/*global Ext, AS*/

/**
 * Asset info panel.
 *
 * @since 3.6
 */
Ext.define('AS.coreui.view.component.ComponentAssetInfo', {
  extend: 'Ext.Panel',
  alias: 'widget.as-coreui-component-componentassetinfo',
  requires: [
    'AS.I18n',
    'AS.coreui.util.RepositoryUrls',
    'AS.ext.button.Button',
    'AS.view.info.DependencySnippetPanel'
  ],

  mixins: {
    componentUtils: 'AS.coreui.mixin.ComponentUtils'
  },

  autoScroll: true,
  cls: 'as-coreui-component-componentassetinfo',

  dockedItems: {
    xtype: 'as-actions',
    items: [
      {
        xtype: 'as-button',
        text: AS.I18n.get('ComponentDetails_View_Vulnerabilities_Button'),
        iconCls: 'x-fa fa-bug',
        reference: 'viewVulnerabilitiesButton',
        action: 'viewVulnerabilities',
        hidden: true
      },
      {
        xtype: 'as-button',
        text: AS.I18n.get('FolderInfo_Delete_Button'),
        iconCls: 'x-fa fa-trash',
        action: 'deleteFolder',
        hidden: true
      }, {
        xtype: 'as-button',
        text: AS.I18n.get('AssetInfo_Delete_Button'),
        iconCls: 'x-fa fa-trash',
        action: 'deleteAsset',
        hidden: true
      }
    ]
  },

  referenceHolder: true,

  items: [
    {
      xtype: 'as-info-panel',
      reference: 'summaryPanel',
      titled: 'Summary',
      collapsible: true
    },
    {
      xtype: 'as-info-panel',
      reference: 'vulnerabilityPanel',
      titled: AS.I18n.get('Component_Vulnerability_Info_Title'),
      collapsible: true,
      hidden: true
    },
    {
      xtype: 'as-info-dependency-snippet-panel',
      reference: 'dependencySnippetPanel'
    }, {
      xtype: 'panel',
      ui: 'as-inset',
      title: 'Attributes',
      collapsible: true,
      manageHeight: false,
      items: [
        {
          xtype: 'as-coreui-component-assetattributes',
          reference: 'attributesPanel'
        }
      ]
    }
  ],

  savedInfo: {},

  setModel: function(asset, component) {
    var me = this;

    var computedInfo = {},
        contentType = asset.get('contentType'),
        size = asset.get('size'),
        attributesPanel = this.lookup('attributesPanel'),
        titleText;

    this.assetModel = asset;
    this.componentModel = component;

    computedInfo[AS.I18n.get('Assets_Info_Repository')] = asset.get('repositoryName');
    computedInfo[AS.I18n.get('Assets_Info_Format')] = asset.get('format');
    computedInfo[AS.I18n.get('Assets_Info_Group')] = component.get('group');
    computedInfo[AS.I18n.get('Assets_Info_Name')] = component.get('name');
    computedInfo[AS.I18n.get('Assets_Info_Version')] = component.get('version');
    computedInfo[AS.I18n.get('Assets_Info_Path')] = AS.view.info.Entry.rawHtml(
        AS.coreui.util.RepositoryUrls.asRepositoryLink(asset, asset.get('format')));

    // Add format-specific metadata from component attributes
    var format = asset.get('format');
    var componentAttributes = component.get('attributes');

    if (componentAttributes && componentAttributes[format]) {
      var formatAttrs = componentAttributes[format];

      // Add description if available
      if (formatAttrs.description) {
        computedInfo['Description'] = formatAttrs.description;
      }

      // Add authors if available
      if (formatAttrs.authors) {
        var authorsDisplay = Ext.isArray(formatAttrs.authors)
          ? formatAttrs.authors.join(', ')
          : formatAttrs.authors;
        computedInfo['Authors'] = authorsDisplay;
      }

      // Add tags if available
      if (formatAttrs.tags) {
        var tagsDisplay = Ext.isArray(formatAttrs.tags)
          ? formatAttrs.tags.join(', ')
          : formatAttrs.tags;
        computedInfo['Tags'] = tagsDisplay;
      }

      // Add license if available
      if (formatAttrs.license) {
        var licenseDisplay = Ext.isArray(formatAttrs.license)
          ? formatAttrs.license.join(', ')
          : formatAttrs.license;
        computedInfo['License'] = licenseDisplay;
      }
    }

    computedInfo[AS.I18n.get('Assets_Info_ContentType')] = contentType || 'unknown';
    computedInfo[AS.I18n.get('Assets_Info_FileSize')] = Ext.util.Format.fileSize(size);
    computedInfo[AS.I18n.get('Assets_Info_Blob_Created')] = asset.get('blobCreated');
    computedInfo[AS.I18n.get('Assets_Info_Blob_Updated')] = asset.get('blobUpdated');

    if (asset.get('downloadCount')) {
      computedInfo[AS.I18n.get('Assets_Info_Downloaded_Count')] = asset.get('downloadCount') + ' '
          + AS.I18n.get('Assets_Info_Downloaded_Unit');
    }

    computedInfo[AS.I18n.get('Assets_Info_Last_Downloaded')] =
        me.mixins.componentUtils.getLastDownloadDateForDisplay(asset);
    computedInfo[AS.I18n.get('Assets_Info_Locally_Cached')] = contentType !== 'unknown' && size > 0;
    computedInfo[AS.I18n.get('Assets_Info_BlobRef')] = asset.get('blobRef');
    computedInfo[AS.I18n.get('Assets_Info_ContainingRepositoryName')] = asset.get('containingRepositoryName');

    if (asset.get('createdBy')) {
      computedInfo[AS.I18n.get('Assets_Info_UploadedBy')] = asset.get('createdBy');
    }
    if (asset.get('createdByIp')) {
      computedInfo[AS.I18n.get('Assets_Info_UploadedIp')] = asset.get('createdByIp');
    }

    if (attributesPanel) {
      attributesPanel.setAssetModel(asset);
    }

    this.showInfo(Ext.apply({}, this.savedInfo, computedInfo));

    titleText = asset.get('name');
    this.setTitle({
      text: titleText,
      listeners: {
        destroy: function(me) {
          Ext.tip.QuickTipManager.unregister(me.getId());
        }
      }
    });

    // title is already rendered at this point so can't use afterrender event here
    Ext.tip.QuickTipManager.register({
      target: this.down('title').getId(),
      text: titleText
    });

    this.fireEvent('updated', this, asset, component);
  },

  setInfo: function(section, key, value) {
    this.savedInfo[key] = value;
  },

  showInfo: function(info) {
    var summaryPanel = this.lookup('summaryPanel');
    if (summaryPanel) {
      summaryPanel.showInfo(info);
    }
  },

  getDependencySnippetPanel: function() {
    return this.lookup('dependencySnippetPanel');
  },

  getVulnerabilityPanel: function() {
    return this.lookup('vulnerabilityPanel');
  },

  getViewVulnerabilitiesButton: function() {
    return this.lookup('viewVulnerabilitiesButton');
  }
});
