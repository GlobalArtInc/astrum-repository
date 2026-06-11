
/*global Ext*/

/**
 * Main uber mode panel.
 *
 * @since 3.0
 */
Ext.define('NX.view.Main', {
  extend: 'Ext.panel.Panel',
  alias: 'widget.nx-main',
  requires: [
    'NX.I18n',
    'NX.Icons',
    'NX.view.header.QuickSearch',
    'Ext.button.Button',
    'NX.view.UpgradeAlert',
    'NX.view.UpgradeModal',
    'NX.view.CEBanners',
    'NX.view.TelemetryWarningBanner'
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
            xtype: 'nx-header-panel'
          },
          {
            xtype: 'nx-telemetry-warning-banner',
            hidden: false
          }
        ],
        region: 'north',
        collapsible: false
      },

      {
        xtype: 'nx-feature-menu',
        region: 'west',
        border: false,
        resizable: true,
        resizeHandles: 'e'
      },

      {
        xtype: 'nx-feature-content',
        region: 'center',
        border: true
      },

      {
        xtype: 'nx-component-upgrade-modal',
        region: 'south',
        hidden: true
      },

      {
        xtype: 'nx-footer',
        region: 'south',
        hidden: true
      },

      {
        xtype: 'nx-component-upgrade-alert',
        region: 'south',
        hidden: true
      },

      {
        xtype: 'nx-component-ce-banners',
        region: 'north',
        hidden: true
      },

      {
        xtype: 'nx-dev-panel',
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

    me.down('nx-header-panel>toolbar').add([
      // 2x pad
      ' ', ' ',
      {
        xtype: 'nx-header-mode',
        name: 'browse',
        title: NX.I18n.get('Header_BrowseMode_Title'),
        tooltip: NX.I18n.get('Header_BrowseMode_Tooltip'),
        iconCls: 'x-fa fa-cube',
        autoHide: true,
        collapseMenu: true
      },
      {
        xtype: 'nx-header-mode',
        name: 'admin',
        title: NX.I18n.get('Header_AdminMode_Title'),
        tooltip: NX.I18n.get('Header_AdminMode_Tooltip'),
        iconCls: 'x-fa fa-cog',
        autoHide: true,
        collapseMenu: false
      },
      ' ',
      {xtype: 'nx-header-quicksearch', hidden: true},
      '->',
      {
        id: 'nx-health-check-warnings',
        xtype: 'button',
        name: 'metric-health',
        tooltip: NX.I18n.get('Header_Health_Tooltip'),
        iconCls: 'x-fa fa-check-circle',
        autoHide: true,
        hidden: true,
        collapseMenu: false,
        ui: 'nx-mode',
        cls: ['nx-health-button-green', 'nx-modebutton'],
        handler: function() {
          NX.Bookmarks.navigateTo(NX.Bookmarks.fromToken('admin/support/status'));
        }
      },
      {xtype: 'nx-header-refresh', ui: 'nx-header'},
      {xtype: 'nx-header-help', ui: 'nx-header'},
      {
        xtype: 'nx-header-mode',
        name: 'user',
        title: NX.I18n.get('Header_UserMode_Title'),
        iconCls: 'x-fa fa-user',
        autoHide: false,
        collapseMenu: false
      },
      {xtype: 'nx-header-signin', ui: 'nx-header'},
      {xtype: 'nx-header-signout', ui: 'nx-header'}
    ]);
  }

});
