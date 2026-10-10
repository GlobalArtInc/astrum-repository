
/*global Ext*/

/**
 * Abstract change order window.
 *
 * @since 3.0
 */
Ext.define('AS.view.ChangeOrderWindow', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-changeorderwindow',
  requires: [
    'AS.ext.form.field.ItemOrderer',
    'AS.I18n'
  ],
  ui: 'as-inset',

  displayField: 'name',
  valueField: 'id',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.setWidth(AS.view.ModalDialog.MEDIUM_MODAL);

    me.items = {
      xtype: 'form',
      items: {
        xtype: 'as-itemorderer',
        store: me.store,
        displayField: me.displayField,
        valueField: me.valueField,
        delimiter: null,
        height: 400,
        width: 400
      },
      buttonAlign: 'left',
      buttons: [
        { text: AS.I18n.get('ChangeOrderWindow_Submit_Button'), action: 'save', formBind: true, ui: 'as-primary' },
        { text: AS.I18n.get('ChangeOrderWindow_Cancel_Button'), handler: function () {
          this.up('window').close();
        }}
      ]
    };

    me.callParent();
  }

});
