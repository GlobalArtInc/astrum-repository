
/*global Ext, AS*/

/**
 * SMTP System Settings form.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.system.SmtpSettings', {
  extend: 'AS.view.SettingsPanel',
  alias: 'widget.as-coreui-system-smtp-settings',
  requires: [
    'AS.Conditions',
    'AS.I18n',
    'AS.ext.form.field.Hostname'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.settingsForm = [
      {
        xtype: 'as-settingsform',
        settingsFormSuccessMessage: AS.I18n.get('System_SmtpSettings_Update_Success'),
        api: {
          load: 'AS.direct.coreui_Email.read',
          submit: 'AS.direct.coreui_Email.update'
        },
        editableCondition: AS.Conditions.isPermitted('astrum:settings:update'),
        editableMarker: AS.I18n.get('System_SmtpSettings_Update_Error'),

        items: [
          {
            xtype: 'checkbox',
            name: 'enabled',
            fieldLabel: AS.I18n.get('System_SmtpSettings_Enabled_FieldLabel')
          },
          {
            xtype: 'as-hostname',
            name: 'host',
            itemId: 'host',
            fieldLabel: AS.I18n.get('System_SmtpSettings_Host_FieldLabel')
          },
          {
            xtype: 'numberfield',
            name: 'port',
            itemId: 'port',
            fieldLabel: AS.I18n.get('System_SmtpSettings_Port_FieldLabel'),
            minValue: 1,
            maxValue: 65536,
            allowDecimals: false,
            allowExponential: false,
            useTrustStore: function (field) {
              var form = field.up('form');
              return {
                name: 'astrumTrustStoreEnabled',
                host: form.down('#host'),
                port: form.down('#port')
              };
            }
          },
          {
            xtype: 'textfield',
            name: 'username',
            fieldLabel: AS.I18n.get('System_SmtpSettings_Username_FieldLabel'),
            allowBlank: true,
            inputAttrTpl: 'autocomplete="new-username"'
          },
          {
            xtype: 'as-password',
            name: 'password',
            fieldLabel: AS.I18n.get('System_SmtpSettings_Password_FieldLabel'),
            allowBlank: true,
            inputAttrTpl: 'autocomplete="new-password"'
          },
          {
            xtype: 'as-email',
            name: 'fromAddress',
            fieldLabel: AS.I18n.get('System_SmtpSettings_FromAddress_FieldLabel')
          },
          {
            xtype: 'textfield',
            name: 'subjectPrefix',
            fieldLabel: AS.I18n.get('System_SmtpSettings_SubjectPrefix_FieldLabel'),
            allowBlank: true
          },
          {
            xtype: 'checkboxgroup',
            fieldLabel: AS.I18n.get('System_SmtpSettings_SslTlsSection_FieldLabel'),
            columns: 1,
            allowBlank: true,
            items: [
              {
                xtype: 'checkbox',
                name: 'startTlsEnabled',
                boxLabel: AS.I18n.get('System_SmtpSettings_StartTlsEnabled_FieldLabel')
              },
              {
                xtype: 'checkbox',
                name: 'startTlsRequired',
                boxLabel: AS.I18n.get('System_SmtpSettings_StartTlsRequired_FieldLabel')
              },
              {
                xtype: 'checkbox',
                name: 'sslOnConnectEnabled',
                boxLabel: AS.I18n.get('System_SmtpSettings_SslOnConnectEnabled_FieldLabel')
              },
              {
                xtype: 'checkbox',
                name: 'sslCheckServerIdentityEnabled',
                boxLabel: AS.I18n.get('System_SmtpSettings_SslCheckServerIdentityEnabled_FieldLabel')
              }
            ]
          }
        ]
      }
    ];

    me.callParent();

    me.down('as-settingsform').getDockedItems('toolbar[dock="bottom"]')[0].add({
      xtype: 'button',
      text: AS.I18n.get('System_SmtpSettings_VerifyServer_Button'),
      formBind: true,
      action: 'verify',
      iconCls: 'x-fa fa-envelope'
    });
  }
});
