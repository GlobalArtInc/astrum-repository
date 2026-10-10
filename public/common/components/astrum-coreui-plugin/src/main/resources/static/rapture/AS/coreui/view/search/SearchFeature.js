
/*global Ext, AS*/

/**
 * Search feature.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.search.SearchFeature', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-searchfeature',

  cls: 'as-coreui-searchfeature',
  iconCls: 'x-fa fa-search',

  requires: [
    'AS.State'
  ],

  initComponent: function() {
    var me = this,
        searchItem = this.getSqlSearchItem();

    me.masters = [
      {
        // FIXME: change to container
        xtype: 'panel',
        layout: {
          type: 'vbox',
          align: 'stretch',
          pack: 'start'
        },
        items: [
          {
            // FIXME: change to container
            xtype: 'panel',
            cls: 'criteria',
            itemId: 'criteria',

            header: false,
            layout: 'column'

            // disable saving for now
            //tbar: [
            //  { xtype: 'button', text: 'Save', glyph: 'xf0c7@FontAwesome', action: 'save' },
            //],
          },
          {
            xtype: 'panel',
            itemId: 'info',
            ui: 'as-info-message',
            cls: 'info-message',
            iconCls: AS.Icons.cls('message-primary', 'x16'),
            hidden: true
          },
          {
            xtype: 'panel',
            itemId: 'warning',
            ui: 'as-info-message',
            cls: 'warning-message',
            iconCls: AS.Icons.cls('message-danger', 'x16'),
            hidden: true
          },
          // depends on SQL search to show corresponding message
          searchItem
        ]
      },
      {
        // FIXME: change to container?
        xtype: 'panel',
        layout: {
          type: 'vbox',
          align: 'stretch',
          pack: 'start'
        },
        items: [
          {
            xtype: 'as-coreui-component-details'
          },
          {
            xtype: 'as-coreui-component-asset-list',
            flex: 1
          }
        ]
      }
    ];

    me.detail = {
      xtype: 'as-coreui-component-assetcontainer',
      header: false,
      flex: 1
    };

    me.callParent();
  },

  getSqlSearchItem: function() {
    var isKeywordSearch = this.searchFilter.getId() === 'keyword';

    return {
      xtype: 'panel',
      cls: 'searchInfo',
      itemId: 'searchInfo',
      title:
          {
            iconCls: 'x-fa fa-info-circle',
            text: isKeywordSearch ? AS.I18n.get('Search_KeywordSearchRestrictions')
                : AS.I18n.get('Search_SearchRestrictions')
          },
      layout: 'fit',
      flex: 1,
      items: [
        {
          xtype: 'as-coreui-search-result-list',
          cls: 'as-search-result-list',
          flex: 1,
          header: false
        }
      ]
    }
  }

});
