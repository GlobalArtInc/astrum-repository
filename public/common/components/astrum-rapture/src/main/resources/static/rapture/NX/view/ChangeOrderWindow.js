
/*global Ext*/

/**
 * Abstract change order window.
 *
 * @since 3.0
 */
Ext.define('NX.view.ChangeOrderWindow', {
  extend: 'NX.view.ModalDialog',
  alias: 'widget.nx-changeorderwindow',
  requires: [
    'NX.ext.form.field.ItemOrderer',
    'NX.I18n'
  ],
  ui: 'nx-inset',

  displayField: 'name',
  valueField: 'id',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.setWidth(NX.view.ModalDialog.MEDIUM_MODAL);

    me.items = {
      xtype: 'form',
      items: {
        xtype: 'nx-itemorderer',
        store: me.store,
        displayField: me.displayField,
        valueField: me.valueField,
        delimiter: null,
        height: 400,
        width: 400
      },
      buttonAlign: 'left',
      buttons: [
        { text: NX.I18n.get('ChangeOrderWindow_Submit_Button'), action: 'save', formBind: true, ui: 'nx-primary' },
        { text: NX.I18n.get('ChangeOrderWindow_Cancel_Button'), handler: function () {
          this.up('window').close();
        }}
      ]
    };

    me.callParent();
  }

});
