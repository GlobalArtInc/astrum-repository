
/*global Ext, NX*/

/**
 * CLM test results window (shows a list of applications).
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.clm.ClmSettingsTestResults', {
  extend: 'NX.view.ModalDialog',
  alias: 'widget.nx-coreui-clm-settings-testresults',
  requires: [
    'Ext.data.JsonStore',
    'NX.I18n'
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

    me.title = NX.I18n.get('Clm_SettingsTestResults_Title');

    me.layout = 'fit';
    me.closable = true;
    me.autoShow = true;
    me.modal = true;
    me.constrain = true;

    me.buttonAlign = 'left';
    me.buttons = [
      { text: NX.I18n.get('Button_Close'), handler: function () {
        this.up('window').close();
      }}
    ];

    me.items = {
      xtype: 'grid',
      emptyText: NX.I18n.get('Clm_SettingsTestResults_EmptyText'),
      columns: [
        { header: NX.I18n.get('Clm_SettingsTestResults_Id_Header'), dataIndex: 'id', flex: 1, renderer: Ext.htmlEncode },
        { header: NX.I18n.get('Clm_SettingsTestResults_Name_Header'), dataIndex: 'name', flex: 1, renderer: Ext.htmlEncode }
      ],
      store: Ext.create('Ext.data.JsonStore', {
        fields: ['id', 'name'],
        data: me.applications
      })
    };

    me.width = NX.view.ModalDialog.LARGE_MODAL;
    me.maxHeight = me.height = Ext.getBody().getViewSize().height - 100;

    me.callParent();
    me.center();
  }

});
