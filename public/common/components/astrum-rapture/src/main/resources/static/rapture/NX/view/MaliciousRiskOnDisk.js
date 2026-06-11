
Ext.define('NX.view.MaliciousRiskOnDisk', {
  extend: 'Ext.container.Container',
  alias: 'widget.nx-component-malicious-risk-on-disk',
  requires: [
    'NX.view.SecondaryContainer'
  ],

  /**
   * @override
   */
  initComponent: function() {
    const me = this;

    me.items = [
      {
        xtype: 'nx-secondary-container',
        reactView: window.ReactComponents.MaliciousRiskOnDisk,
        reactViewProps: {
          rerender: function(count, hide) {
            if (count === 0 || hide) {
              me.hide();
            } else {
              me.show();
            }
          },
          toggle: me.toggle.bind(me),
        }
      }
    ];

    me.show();
    me.callParent();
  },

  toggle: function(height) {
    const me = this;
    const container = me.down('nx-secondary-container');

    if (container) {
      me.setHeight(height);
      container.setHeight(height);
      container.updateLayout();
    }
  },

  rerender: function() {
    const me = this;
    const container = me.down('nx-secondary-container');

    if (me.rendered) {
      container.refresh();
    }
  }
});
