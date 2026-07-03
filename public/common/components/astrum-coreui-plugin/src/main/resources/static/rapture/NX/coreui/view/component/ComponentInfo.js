
/*global Ext, NX*/

/**
 * Component info panel.
 *
 * @since 3.4
 */
Ext.define('NX.coreui.view.component.ComponentInfo', {
  extend: 'Ext.Panel',
  alias: 'widget.nx-coreui-component-componentinfo',
  cls: 'nx-coreui-component-componentinfo',
  requires: [
    'NX.I18n',
    'NX.coreui.util.RepositoryUrls',
    'NX.ext.button.Button',
    'NX.view.info.DependencySnippetPanel',
    'NX.coreui.view.component.AssetAttributes'
  ],
  dockedItems: {
    xtype: 'nx-actions',
    dock: 'top',
    items: [
      {
        xtype: 'nx-button',
        text: NX.I18n.get('ComponentDetails_View_Vulnerabilities_Button'),
        iconCls: 'x-fa fa-bug',
        reference: 'viewVulnerabilitiesButton',
        action: 'viewVulnerabilities',
        hidden: true
      },
      {
        xtype: 'nx-button',
        text: NX.I18n.get('ComponentDetails_Delete_Button'),
        iconCls: 'x-fa fa-trash',
        action: 'deleteComponent',
        hidden: true
      }
    ]
  },

  referenceHolder: true,

  items: [
    {
      xtype: 'nx-info-panel',
      reference: 'summaryPanel',
      titled: NX.I18n.get('Component_AssetInfo_Info_Title'),
      collapsible: true
    },
    {
      xtype: 'nx-info-panel',
      reference: 'vulnerabilityPanel',
      titled: NX.I18n.get('Component_Vulnerability_Info_Title'),
      collapsible: true,
      hidden: true
    },
    {
      xtype: 'nx-info-dependency-snippet-panel',
      reference: 'dependencySnippetPanel'
    },
    {
      xtype: 'panel',
      ui: 'nx-inset',
      title: 'Attributes',
      collapsible: true,
      manageHeight: false,
      items: [
        {
          xtype: 'nx-coreui-component-assetattributes',
          reference: 'attributesPanel'
        }
      ]
    }
  ],
  autoScroll: true,
  summary: {},

  setModel: function(componentModel) {
    var me = this,
        summary = this.summary,
        componentName = componentModel.get('name'),
        attributesPanel = this.lookup('attributesPanel');

    this.componentModel = componentModel;

    summary[NX.I18n.get('Search_Assets_Repository')] = componentModel.get('repositoryName');
    summary[NX.I18n.get('Search_Assets_Format')] = componentModel.get('format');
    summary[NX.I18n.get('Search_Assets_Group')] = componentModel.get('group');
    summary[NX.I18n.get('Search_Assets_Name')] = componentModel.get('name');
    summary[NX.I18n.get('Search_Assets_Version')] = componentModel.get('version');

    me.showInfo();

    if (attributesPanel) {
      attributesPanel.setAssetModel(componentModel);
    }

    Ext.tip.QuickTipManager.unregister(me.down('title').getId());
    Ext.tip.QuickTipManager.register({
      target: me.down('title').getId(),
      text: componentName
    });

    me.fireEvent('updated', me, me.componentModel);
  },

  setInfo: function(section, key, value) {
    this.summary[key] = value;
  },

  showInfo: function() {
    var summaryPanel = this.lookup('summaryPanel');
    if (summaryPanel) {
      summaryPanel.showInfo(this.summary);
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
