
/*global Ext*/

Ext.define('NX.view.TelemetryWarningBanner', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-telemetry-warning-banner',
  requires: [
    'NX.view.SecondaryContainer'
  ],

  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'nx-secondary-container',
        reactView: window.ReactComponents ? window.ReactComponents.TelemetryWarningBanner : null,
        reactViewProps: {}
      }
    ];

    me.callParent();
  }
});
