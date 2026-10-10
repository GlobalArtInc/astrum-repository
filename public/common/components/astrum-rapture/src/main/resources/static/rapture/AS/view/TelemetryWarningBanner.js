
/*global Ext*/

Ext.define('AS.view.TelemetryWarningBanner', {
  extend: 'Ext.container.Container',
  alias: 'widget.as-telemetry-warning-banner',
  requires: [
    'AS.view.SecondaryContainer'
  ],

  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'as-secondary-container',
        reactView: window.ReactComponents ? window.ReactComponents.TelemetryWarningBanner : null,
        reactViewProps: {}
      }
    ];

    me.callParent();
  }
});
