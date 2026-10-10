
/*global Ext, AS*/

/**
 * LDAP Server User & Group test results window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerUserAndGroupMappingTestResults', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-coreui-ldapserver-userandgroup-testresults',
  requires: [
    'Ext.data.JsonStore',
    'AS.I18n'
  ],

  /**
   * @cfg json array of users (as returned by checking the user mapping)
   */
  config: {
    mappedUsers: undefined
  },

  initComponent: function () {
    var me = this;

    me.title = AS.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_Title');
    me.layout = 'fit';
    me.closeable = true;
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
      columns: [
        { header: AS.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_ID_Header'), dataIndex: 'username', flex: 1, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_Name_Header'), dataIndex: 'realName', flex: 1, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_Email_Header'), dataIndex: 'email', width: 250, renderer: Ext.htmlEncode },
        { header: AS.I18n.get('Ldap_LdapServerUserAndGroupMappingTestResults_Roles_Header'), dataIndex: 'membership', flex: 3, renderer: Ext.htmlEncode }
      ],
      store: Ext.create('Ext.data.JsonStore', {
        fields: ['username', 'realName', 'email', 'membership'],
        data: me.getMappedUsers()
      })
    };

    me.width = AS.view.ModalDialog.LARGE_MODAL;
    me.maxHeight = me.height = Ext.getBody().getViewSize().height - 100;

    me.callParent();
    me.center();
  }

});
