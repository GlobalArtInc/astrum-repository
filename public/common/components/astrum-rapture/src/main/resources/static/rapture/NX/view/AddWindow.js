
/*global Ext*/

/**
 * Abstract add window.
 *
 * @since 3.0
 */
Ext.define('NX.view.AddWindow', {
  extend: 'Ext.window.Window',
  alias: 'widget.nx-addwindow',
  requires: [
    'NX.I18n'
  ],

  layout: 'fit',
  autoShow: true,
  modal: true,
  constrain: true,
  width: 630,
  minWidth: 630,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    if (Ext.isDefined(me.items) && !Ext.isArray(me.items)) {
      if (!me.items.buttons) {
        me.items.buttons = [
          { text: NX.I18n.get('Add_Submit_Button'), action: 'add', formBind: true, ui: 'nx-primary', bindToEnter: me.items.settingsFormSubmitOnEnter },
          { text: NX.I18n.get('Add_Cancel_Button'), handler: function () {
            this.up('window').close();
          }}
        ];
      }
    }

    me.maxHeight = Ext.getBody().getViewSize().height - 100;

    me.callParent();
  }

});
