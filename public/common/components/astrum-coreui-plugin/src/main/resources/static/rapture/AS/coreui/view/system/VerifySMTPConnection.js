
/*global Ext, AS*/

/**
 * Verify SMTP connection window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.system.VerifySMTPConnection', {
  extend: 'Ext.window.Window',
  alias: 'widget.as-coreui-system-verifysmtpconnection',
  requires: [
    'AS.Icons',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.ui = 'as-inset';
    me.title = AS.I18n.get('System_VerifySmtpConnection_VerifyServer_Title');
    me.defaultFocus = 'email';
    me.layout = 'fit';
    me.autoShow = true;
    me.constrain = true;
    me.resizable = false;
    me.width = 500;
    me.border = false;
    me.modal = true;

    me.items = [
      {
        xtype: 'form',
        defaults: {
          labelSeparator: '',
          labelWidth: 40,
          labelAlign: 'right',
          anchor: '100%'
        },
        items: [
          {
            xtype: 'panel',
            layout: 'hbox',
            style: {
              // FIXME: sort out common style here for dialogs
              marginBottom: '10px'
            },
            items: [
              { xtype: 'component', html: AS.Icons.img('verifysmtpconnection', 'x32') },
              {
                xtype: 'label',
                html: AS.I18n.get('System_VerifySmtpConnection_HelpText'),
                margin: '0 0 0 5'
              }
            ]
          },
          {
            xtype: 'as-email',
            name: 'email',
            itemId: 'email',
            fieldLabel: 'E-mail',
            allowBlank: false,
            // allow cancel to be clicked w/o validating this to be non-blank
            validateOnBlur: false
          }
        ],

        buttonAlign: 'left',
        buttons: [
          {
            text: 'Verify',
            action: 'verify',
            formBind: true,
            bindToEnter: true,
            ui: 'as-primary',
            iconCls: 'x-fa fa-envelope'
          },
          {
            text: 'Cancel',
            handler: function () {
              this.up('window').close();
            }
          }
        ]
      }
    ];

    me.callParent();
  }
});
