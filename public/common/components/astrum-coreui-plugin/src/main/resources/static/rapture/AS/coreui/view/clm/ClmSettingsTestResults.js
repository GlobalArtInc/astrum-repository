
/*global Ext, AS*/

/**
 * CLM test results window (shows a list of applications).
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.clm.ClmSettingsTestResults', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-coreui-clm-settings-testresults',
  requires: [
    'Ext.data.JsonStore',
    'AS.I18n'
  ],

  /**
   * @cfg json array of applications (as returned by checking the connection)
   */
  applications: undefined,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = AS.I18n.get('Clm_SettingsTestResults_Title');

    me.layout = 'fit';
    me.closable = true;
    me.autoShow = true;
    me.modal = true;
    me.constrain = true;

    me.buttonAlign = 'left';
    me.buttons = [
      { text: AS.I18n.get('Button_Close'), handler: function () {
        this.up('window').close();
      }}
    ];

    me.items = {
      xtype: 'grid',
      emptyText: AS.I18n.get('Clm_SettingsTestResults_EmptyText'),
      columns: [
        { header: AS.I18n.get('Clm_SettingsTestResults_Id_Header'), dataIndex: 'id', flex: 1, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Clm_SettingsTestResults_Name_Header'), dataIndex: 'name', flex: 1, renderer: Ext.htmlEncode }
      ],
      store: Ext.create('Ext.data.JsonStore', {
        fields: ['id', 'name'],
        data: me.applications
      })
    };

    me.width = AS.view.ModalDialog.LARGE_MODAL;
    me.maxHeight = me.height = Ext.getBody().getViewSize().height - 100;

    me.callParent();
    me.center();
  }

});
