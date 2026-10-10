
/*global Ext*/

/**
 * Expire session window.
 *
 * @since 3.0
 */
Ext.define('AS.view.ExpireSession', {
  extend: 'AS.view.ModalDialog',
  requires: [
    'AS.I18n'
  ],
  alias: 'widget.as-expire-session',

  cls: 'as-expire-session',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = AS.I18n.get('ExpireSession_Title');

    me.setWidth(AS.view.ModalDialog.MEDIUM_MODAL);

    Ext.apply(me, {
      items: [
        {
          xtype: 'label',
          // FIXME: Why is this using global 'id'?
          id: 'expire',
          text: AS.I18n.get('ExpireSession_Help_Text')
        }
      ],
      buttonAlign: 'left',
      buttons: [
        { text: AS.I18n.get('ExpireSession_Cancel_Button'), action: 'cancel' },
        {
          text: AS.I18n.get('ExpireSession_SignIn_Button'),
          action: 'signin',
          hidden: true,
          itemId: 'expiredSignIn',
          ui: 'as-primary',
          handler: function() {
            // FIXME: simplify, me.close()
            this.up('as-expire-session').close();
          }
        },
        {
          text: AS.I18n.get('Button_Close'),
          action: 'close',
          hidden: true,
          handler: function() {
            // FIXME: simplify, me.close()
            this.up('as-expire-session').close();
          }
        }
      ]
    });

    me.callParent();
  },

  /**
   * Check to see if the dialog is showing that it is expired.
   *
   * @public
   * @returns {boolean}
   */
  sessionExpired: function() {
    return this.down('#expiredSignIn').isVisible();
  }

});
