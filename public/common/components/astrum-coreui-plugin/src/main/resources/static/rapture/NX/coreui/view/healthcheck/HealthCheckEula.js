
/*global Ext, NX*/

/**
 * Health Check EULA window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.healthcheck.HealthCheckEula', {
  extend: 'Ext.window.Window',
  alias: 'widget.nx-coreui-healthcheck-eula',
  requires: [
    'NX.util.Url'
  ],

  title: 'Astrum IQ Server Terms of Use',

  layout: 'fit',
  autoShow: true,
  modal: true,
  constrain: true,
  width: 640,
  height: 500,

  acceptFn: undefined,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.items = {
      xtype: 'box',
      autoEl: {
        tag: 'iframe',
        src: NX.util.Url.urlOf('/static/healthcheck-tos.html')
      }
    };

    me.dockedItems = [
      {
        xtype: 'toolbar',
        dock: 'bottom',
        ui: 'footer',
        items: [
          { xtype: 'button', text: 'I Accept', action: 'agree', formBind: true, ui: 'nx-primary', handler: function () {
            var win = this.up('window');

            win.close();
            if (win.acceptFn) {
              win.acceptFn.call();
            }
          }},
          { xtype: 'button', text: 'I Do Not Accept', handler: function () {
            this.up('window').close();
          }},
          '->',
          { xtype: 'component', html: '<a href="' + NX.util.Url.urlOf('/static/healthcheck-tos.html') +
              '" target="_new">Download a copy of the license.</a>'
          }
        ]
      }
    ];

    me.callParent();
  }
});
