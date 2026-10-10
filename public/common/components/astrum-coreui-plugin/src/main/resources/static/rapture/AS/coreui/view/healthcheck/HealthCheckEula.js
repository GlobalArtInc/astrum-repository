
/*global Ext, AS*/

/**
 * Health Check EULA window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.healthcheck.HealthCheckEula', {
  extend: 'Ext.window.Window',
  alias: 'widget.as-coreui-healthcheck-eula',
  requires: [
    'AS.util.Url'
  ],

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
    this.title = AS.I18n.get('HealthCheckEula_Title');
    var me = this;

    me.items = {
      xtype: 'box',
      autoEl: {
        tag: 'iframe',
        src: AS.util.Url.urlOf('/static/healthcheck-tos.html')
      }
    };

    me.dockedItems = [
      {
        xtype: 'toolbar',
        dock: 'bottom',
        ui: 'footer',
        items: [
          { xtype: 'button', text: AS.I18n.get('HealthCheckEula_Accept_Button'), action: 'agree', formBind: true, ui: 'as-primary', handler: function () {
            var win = this.up('window');

            win.close();
            if (win.acceptFn) {
              win.acceptFn.call();
            }
          }},
          { xtype: 'button', text: AS.I18n.get('HealthCheckEula_Decline_Button'), handler: function () {
            this.up('window').close();
          }},
          '->',
          { xtype: 'component', html: '<a href="' + AS.util.Url.urlOf('/static/healthcheck-tos.html') +
              '" target="_new">' + AS.I18n.get('HealthCheckEula_Download_Link') + '</a>'
          }
        ]
      }
    ];

    me.callParent();
  }
});
