
/*global Ext, NX*/

/**
 * Verify SMTP connection window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.system.VerifySMTPConnection', {
  extend: 'Ext.window.Window',
  alias: 'widget.nx-coreui-system-verifysmtpconnection',
  requires: [
    'NX.Icons',
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.ui = 'nx-inset';
    me.title = NX.I18n.get('System_VerifySmtpConnection_VerifyServer_Title');
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
              { xtype: 'component', html: NX.Icons.img('verifysmtpconnection', 'x32') },
              {
                xtype: 'label',
                html: NX.I18n.get('System_VerifySmtpConnection_HelpText'),
                margin: '0 0 0 5'
              }
            ]
          },
          {
            xtype: 'nx-email',
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
            ui: 'nx-primary',
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
