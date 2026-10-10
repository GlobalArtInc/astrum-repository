
/*global Ext*/

/**
 * Dependency snippet info panel.
 *
 * @since 3.15
 */
Ext.define('AS.view.info.DependencySnippetPanel', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-info-dependency-snippet-panel',

  requires: [
      'AS.I18n',
      'AS.util.Clipboard'
  ],

  cls: 'as-info-dependency-snippet-panel',

  framed: true,
  scrollable: true,
  header: false,

  /**
   * @private
   */
  initComponent: function() {
    this.items = {
      xtype: 'panel',
      ui: 'as-inset',
      title: AS.I18n.get('DependencySnippetPanel_Title'),
      collapsible: this.collapsible === undefined ? true : this.collapsible,

      items: {
        xtype: 'panel',
        ui: 'as-subsection',
        frame: true,
        layout: 'vbox',
        items: [
          {
            xtype: 'container',
            layout: 'hbox',
            flex: 1,
            height: "auto",
            width: "100%",
            items: [
              {
                xtype: 'combo',
                name: 'toolCombo',
                editable: false,
                store: [],
                queryMode: 'local',
                flex: 1,
                listeners: {
                  change: this.showSnippetText.bind(this)
                }
              }, {
                xtype: 'button',
                action: 'copySnippet',
                margin: '0 0 0 24px',
                tooltip: AS.I18n.get('DependencySnippetPanel_Copy_Button_Tooltip'),
                iconCls: 'x-fa fa-copy',
                listeners: {
                  click: this.onCopyClick.bind(this)
                }
              }
            ]
          }, {
            xtype: 'component',
            name: 'snippet',
            editable: false,
            layout: 'vbox',
            flex: 2,
            tpl: '<p class="description">{description}</p>' +
                 '<pre class="snippet-text">{snippetText}</pre>',
            data: {text: ''},
            width: "100%"
          }
        ]
      }
    };

    this.callParent();
  },

  getSnippetComponent: function() {
    return this.down('component[name="snippet"]');
  },

  getStorageKey: function(format) {
    return 'dependency-snippet-panel-' + format + '-tool-displayName';
  },

  getToolComboBox: function() {
    return this.down('combo[name="toolCombo"]');
  },

  setDependencySnippets: function(format, dependencySnippets) {
    var storedDisplayName = Ext.state.Manager.get(this.getStorageKey(format)),
        toolCombo = this.getToolComboBox();

    this.format = format;
    this.dependencySnippetMap = {};

    if (dependencySnippets && dependencySnippets.length > 0) {
      dependencySnippets.forEach(function(snippet) {
        this.dependencySnippetMap[snippet.displayName] = {
          snippetText: snippet.snippetText,
          description: snippet.description
        };
      }, this);
      this.updateSnippetDisplayNames(dependencySnippets);
      if (storedDisplayName && toolCombo.store.findRecord("field1", storedDisplayName)) {
        this.selectSnippet(storedDisplayName);
      } else {
        this.selectSnippet(dependencySnippets[0].displayName);
      }
    } else {
      toolCombo.setStore([]);
      this.getSnippetComponent().update({
        snippetText: '',
        description: ''
      });
    }
  },

  selectSnippet: function(displayName) {
    var toolCombo = this.getToolComboBox(),
        toolComboSelection = toolCombo.getSelection(),
        selectedDisplayName = toolComboSelection && toolComboSelection.get('field1');

    if (selectedDisplayName === displayName) {
      // change event won't get fired so explicitly trigger text update
      this.showSnippetText(toolCombo, displayName);
    } else {
      toolCombo.select(displayName);
    }
  },

  showSnippetText: function(toolCombo, selectedDisplayName) {
    var snippet = this.dependencySnippetMap[selectedDisplayName];

    if (snippet) {
      this.getSnippetComponent().update({
        snippetText: Ext.htmlEncode(snippet.snippetText),
        description: Ext.htmlEncode(snippet.description)
      });
      Ext.state.Manager.set(this.getStorageKey(this.format), selectedDisplayName);
    }

    this.fireEvent('snippetDisplayed', {
      format: this.format,
      snippet: selectedDisplayName
    });
  },

  updateSnippetDisplayNames: function(dependencySnippets) {
    var toolCombo = this.getToolComboBox(),
        displayNames = dependencySnippets.map(function(snippet) {
          return snippet.displayName;
        });

    toolCombo.setStore(displayNames);
  },

  onCopyClick: function() {
    var text = this.getSnippetComponent().getData().snippetText;
    AS.util.Clipboard.copyToClipboard(Ext.htmlDecode(text));
  }
});
