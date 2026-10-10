
/*global Ext, AS*/

/**
 * About window.
 *
 * @since 3.0
 */
Ext.define('AS.view.AboutWindow', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-aboutwindow',
  requires: [
    'AS.I18n',
    'AS.Icons',
    'AS.State',
    'AS.util.Url'
  ],

  cls: 'as-aboutwindow',

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
    me.width = AS.view.ModalDialog.LARGE_MODAL;

    me.title = AS.I18n.get('AboutWindow_Title');

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
            html: AS.Icons.img('new-astrum-black', 'x100')
          },
          {
            xtype: 'as-info',
            itemId: 'aboutInfo',
            flex: 1
          }
        ]
      },
      {
        xtype: 'tabpanel',
        ui: 'as-light',
        flex: 1,
        items: [
          {
            title: AS.I18n.get('AboutWindow_About_Title'),
            xtype: 'uxiframe',
            src: AS.util.Url.urlOf('/COPYRIGHT.html')
          }
        ]
      }
    ];

    me.buttons = [
      { text: AS.I18n.get('Button_Close'), action: 'close', ui: 'as-primary', handler: function () { me.close(); }}
    ];
    me.buttonAlign = 'left';

    me.callParent();

    // populate initial details only if not running in the cloud
    if (!AS.State.getValue('isCloud')) {
      var info = {};
      info[AS.I18n.get('AboutWindow_Version_Label')] = AS.State.getVersion();
      info[AS.I18n.get('AboutWindow_Edition_Label')] = AS.State.getEdition();
      info[AS.I18n.get('AboutWindow_BuildRevision_Label')] = AS.State.getBuildRevision();
      info[AS.I18n.get('AboutWindow_BuildTimestamp_Label')] = AS.State.getBuildTimestamp();
      me.down('#aboutInfo').showInfo(info);
    };
  }
});
