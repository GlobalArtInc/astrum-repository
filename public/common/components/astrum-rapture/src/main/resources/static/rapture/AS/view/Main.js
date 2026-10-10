
/*global Ext*/

/**
 * Main uber mode panel.
 *
 * @since 3.0
 */
Ext.define('AS.view.Main', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.as-main',
  requires: [
    'AS.I18n',
    'AS.Icons',
    'AS.view.header.QuickSearch',
    'Ext.button.Button',
    'AS.view.UpgradeAlert',
    'AS.view.UpgradeModal',
    'AS.view.CEBanners',
    'AS.view.TelemetryWarningBanner'
  ],

  layout: 'border',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = [
      {
        xtype: 'panel',
        layout: {
          type: 'vbox',
          align: 'stretch'
        },
        items: [
          {
            xtype: 'as-header-panel'
          },
          {
            xtype: 'as-telemetry-warning-banner',
            hidden: false
          }
        ],
        region: 'north',
        collapsible: false
      },

      {
        xtype: 'as-feature-menu',
        region: 'west',
        border: false,
        resizable: true,
        resizeHandles: 'e'
      },

      {
        xtype: 'as-feature-content',
        region: 'center',
        border: true
      },

      {
        xtype: 'as-component-upgrade-modal',
        region: 'south',
        hidden: true
      },

      {
        xtype: 'as-footer',
        region: 'south',
        hidden: true
      },

      {
        xtype: 'as-component-upgrade-alert',
        region: 'south',
        hidden: true
      },

      {
        xtype: 'as-component-ce-banners',
        region: 'north',
        hidden: true
      },

      {
        xtype: 'as-dev-panel',
        region: 'south',
        collapsible: true,
        collapsed: true,
        resizable: true,
        resizeHandles: 'n',

        // keep initial constraints to prevent huge panels
        height: 300,

        // default to hidden, only show if debug enabled
        hidden: true
      }
    ];

    me.callParent();

    me.down('as-header-panel>toolbar').add([
      // 2x pad
      ' ', ' ',
      {
        xtype: 'as-header-mode',
        name: 'browse',
        title: AS.I18n.get('Header_BrowseMode_Title'),
        tooltip: AS.I18n.get('Header_BrowseMode_Tooltip'),
        iconCls: 'x-fa fa-cube',
        autoHide: true,
        collapseMenu: true
      },
      {
        xtype: 'as-header-mode',
        name: 'admin',
        title: AS.I18n.get('Header_AdminMode_Title'),
        tooltip: AS.I18n.get('Header_AdminMode_Tooltip'),
        iconCls: 'x-fa fa-cog',
        autoHide: true,
        collapseMenu: false
      },
      ' ',
      {xtype: 'as-header-quicksearch', hidden: true},
      '->',
      {
        id: 'as-health-check-warnings',
        xtype: 'button',
        name: 'metric-health',
        tooltip: AS.I18n.get('Header_Health_Tooltip'),
        iconCls: 'x-fa fa-check-circle',
        autoHide: true,
        hidden: true,
        collapseMenu: false,
        ui: 'as-mode',
        cls: ['as-health-button-green', 'as-modebutton'],
        handler: function() {
          AS.Bookmarks.navigateTo(AS.Bookmarks.fromToken('admin/support/status'));
        }
      },
      {xtype: 'as-header-refresh', ui: 'as-header'},
      {xtype: 'as-header-help', ui: 'as-header'},
      {
        xtype: 'as-header-mode',
        name: 'user',
        title: AS.I18n.get('Header_UserMode_Title'),
        iconCls: 'x-fa fa-user',
        autoHide: false,
        collapseMenu: false
      },
      {xtype: 'as-header-signin', ui: 'as-header'},
      {xtype: 'as-header-signout', ui: 'as-header'}
    ]);
  }

});
