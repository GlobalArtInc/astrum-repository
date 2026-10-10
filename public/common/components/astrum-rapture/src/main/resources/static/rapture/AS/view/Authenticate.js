
/*global Ext, AS*/

/**
 * Authenticate window.
 *
 * @since 3.0
 */
Ext.define('AS.view.Authenticate', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-authenticate',
  requires: [
    'AS.Icons',
    'AS.I18n'
  ],

  cls: 'as-authenticate',

  /**
   * @cfg message Message to be shown
   */
  message: undefined,

  /**
   * @protected
   */
  initComponent: function () {
    var me = this;

    me.ui = 'as-inset';
    me.title = AS.I18n.get('Authenticate_Title');

    me.setWidth(AS.view.ModalDialog.MEDIUM_MODAL);

    if (!me.message) {
      me.message = AS.I18n.get('Authenticate_Help_Text');
    }

    Ext.apply(this, {
      closable:false,
      items: {
        xtype: 'form',
        defaultType: 'textfield',
        defaults: {
          anchor: '100%'
        },
        items: [
          {
            xtype: 'container',
            layout: 'hbox',
            cls: 'message',
            items: [
              { xtype: 'component', html: AS.Icons.img('authenticate', 'x32') },
              { xtype: 'label', height: 48, html: '<div>' + me.message + '</div>' }
            ]
          },
          {
            name: 'username',
            itemId: 'username',
            emptyText: AS.I18n.get('SignIn_Username_Empty'),
            allowBlank: false,
            readOnly: true
          },
          {
            name: 'password',
            itemId: 'password',
            inputType: 'password',
            emptyText: AS.I18n.get('SignIn_Password_Empty'),
            allowBlank: false,
            // allow cancel to be clicked w/o validating this to be non-blank
            validateOnBlur: false
          }
        ],

        buttonAlign: 'left',
        buttons: [
          { text: AS.I18n.get('User_View_Authenticate_Submit_Button'), action: 'authenticate', formBind: true, bindToEnter: true, ui: 'as-primary' },
          {
            text: AS.I18n.get('Authenticate_Cancel_Button'), handler: function() {
              if (!!me.options && Ext.isFunction(me.options.failure)) {
                me.options.failure.call(me.options.failure, me.options);
              }
              me.close();
            }, scope: me
          }
        ]
      }
    });

    me.on({
      resize: function() {
        me.down('#password').focus();
      },
      single: true
    });

    me.callParent();
  }

});
