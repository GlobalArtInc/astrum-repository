
/*global Ext, NX*/

/**
 * About window.
 *
 * @since 3.0
 */
Ext.define('NX.view.AboutWindow', {
  extend: 'NX.view.ModalDialog',
  alias: 'widget.nx-aboutwindow',
  requires: [
    'NX.I18n',
    'NX.Icons',
    'NX.State',
    'NX.util.Url'
  ],

  cls: 'nx-aboutwindow',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.layout = {
      type: 'vbox',
      align: 'stretch'
    };

    me.height = 465;
    me.width = NX.view.ModalDialog.LARGE_MODAL;

    me.title = NX.I18n.get('AboutWindow_Title');

    me.items = [
      {
        xtype: 'container',
        cls: 'summary',
        layout: {
          type: 'hbox',
          align: 'stretch'
        },
        items: [
          {
            xtype: 'component',
            cls: 'logo',
            html: NX.Icons.img('new-astrum-black', 'x100')
          },
          {
            xtype: 'nx-info',
            itemId: 'aboutInfo',
            flex: 1
          }
        ]
      },
      {
        xtype: 'tabpanel',
        ui: 'nx-light',
        flex: 1,
        items: [
          {
            title: NX.I18n.get('AboutWindow_About_Title'),
            xtype: 'uxiframe',
            src: NX.util.Url.urlOf('/COPYRIGHT.html')
          }
        ]
      }
    ];

    me.buttons = [
      { text: NX.I18n.get('Button_Close'), action: 'close', ui: 'nx-primary', handler: function () { me.close(); }}
    ];
    me.buttonAlign = 'left';

    me.callParent();

    // populate initial details only if not running in the cloud
    if (!NX.State.getValue('isCloud')) {
      me.down('#aboutInfo').showInfo({
        'Version': NX.State.getVersion(),
        'Edition': NX.State.getEdition(),
        'Build Revision': NX.State.getBuildRevision(),
        'Build Timestamp': NX.State.getBuildTimestamp()
      });
    };
  }
});
