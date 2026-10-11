
/*global Ext, AS*/

/**
 * Component info panel.
 *
 * @since 3.4
 */
Ext.define('AS.coreui.view.component.ComponentInfo', {
  extend: 'Ext.Panel',
  alias: 'widget.as-coreui-component-componentinfo',
  cls: 'as-coreui-component-componentinfo',
  requires: [
    'AS.I18n',
    'AS.coreui.util.RepositoryUrls',
    'AS.ext.button.Button',
    'AS.view.info.DependencySnippetPanel',
    'AS.coreui.view.component.AssetAttributes'
  ],
  dockedItems: {
    xtype: 'as-actions',
    dock: 'top',
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
        text: AS.I18n.get('ComponentDetails_Delete_Button'),
        iconCls: 'x-fa fa-trash',
        action: 'deleteComponent',
        hidden: true
      }
    ]
  },

  referenceHolder: true,

  items: [
    {
      xtype: 'as-info-panel',
      reference: 'summaryPanel',
      titled: AS.I18n.get('Component_AssetInfo_Info_Title'),
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
    },
    {
      xtype: 'panel',
      ui: 'as-inset',
      title: AS.I18n.get('ComponentInfo_Attributes_Title'),
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
  autoScroll: true,
  summary: {},

  setModel: function(componentModel) {
    var me = this,
        summary = this.summary,
        componentName = componentModel.get('name'),
        attributesPanel = this.lookup('attributesPanel');

    this.componentModel = componentModel;

    summary[AS.I18n.get('Search_Assets_Repository')] = componentModel.get('repositoryName');
    summary[AS.I18n.get('Search_Assets_Format')] = componentModel.get('format');
    summary[AS.I18n.get('Search_Assets_Group')] = componentModel.get('group');
    summary[AS.I18n.get('Search_Assets_Name')] = componentModel.get('name');
    summary[AS.I18n.get('Search_Assets_Version')] = componentModel.get('version');
    summary[AS.I18n.get('Assets_Info_Downloaded_Count')] = componentModel.get('downloadCount') !== null
        ? Ext.util.Format.number(componentModel.get('downloadCount'), '0,000')
        : null;

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
