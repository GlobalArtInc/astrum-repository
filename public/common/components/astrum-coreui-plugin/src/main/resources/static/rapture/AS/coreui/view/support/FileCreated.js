
/*global Ext, AS*/

/**
 * File created window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.support.FileCreated', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-coreui-support-filecreated',

  // FIXME: remove use of as-inset
  ui: 'as-inset',

  // FIXME: convert to config{} block
  /**
   * @cfg Icon to show (img)
   */
  fileIcon: undefined,

  /**
   * @cfg Type of file shown
   */
  fileType: undefined,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.setWidth(AS.view.ModalDialog.LARGE_MODAL);

    Ext.apply(me, {
      title: me.title || me.fileType + ' Created',
      items: [
        {
          xtype: 'as-coreui-support-filecreatedform',
          fileIcon: me.fileIcon,
          fileType: me.fileType
        }
      ]
    });

    me.callParent();
  },

  /**
   * Set form values.
   *
   * @public
   */
  setValues: function (values) {
    this.down('as-coreui-support-filecreatedform').getForm().setValues(values);
  },

  /**
   * Get form values.
   *
   * @public
   */
  getValues: function () {
    return this.down('as-coreui-support-filecreatedform').getForm().getValues();
  }
});
