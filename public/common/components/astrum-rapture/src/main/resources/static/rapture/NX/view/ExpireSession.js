
/*global Ext*/

/**
 * Expire session window.
 *
 * @since 3.0
 */
Ext.define('NX.view.ExpireSession', {
  extend: 'NX.view.ModalDialog',
  requires: [
    'NX.I18n'
  ],
  alias: 'widget.nx-expire-session',

  cls: 'nx-expire-session',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.title = NX.I18n.get('ExpireSession_Title');

    me.setWidth(NX.view.ModalDialog.MEDIUM_MODAL);

    Ext.apply(me, {
      items: [
        {
          xtype: 'label',
          // FIXME: Why is this using global 'id'?
          id: 'expire',
          text: NX.I18n.get('ExpireSession_Help_Text')
        }
      ],
      buttonAlign: 'left',
      buttons: [
        { text: NX.I18n.get('ExpireSession_Cancel_Button'), action: 'cancel' },
        {
          text: NX.I18n.get('ExpireSession_SignIn_Button'),
          action: 'signin',
          hidden: true,
          itemId: 'expiredSignIn',
          ui: 'nx-primary',
          handler: function() {
            // FIXME: simplify, me.close()
            this.up('nx-expire-session').close();
          }
        },
        {
          text: NX.I18n.get('Button_Close'),
          action: 'close',
          hidden: true,
          handler: function() {
            // FIXME: simplify, me.close()
            this.up('nx-expire-session').close();
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
