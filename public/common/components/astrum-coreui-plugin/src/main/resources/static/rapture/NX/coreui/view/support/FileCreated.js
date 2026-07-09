
/*global Ext, NX*/

/**
 * File created window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.support.FileCreated', {
  extend: 'NX.view.ModalDialog',
  alias: 'widget.nx-coreui-support-filecreated',

  // FIXME: remove use of nx-inset
  ui: 'nx-inset',

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

    me.setWidth(NX.view.ModalDialog.LARGE_MODAL);

    Ext.apply(me, {
      title: me.title || me.fileType + ' Created',
      items: [
        {
          xtype: 'nx-coreui-support-filecreatedform',
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
    this.down('nx-coreui-support-filecreatedform').getForm().setValues(values);
  },

  /**
   * Get form values.
   *
   * @public
   */
  getValues: function () {
    return this.down('nx-coreui-support-filecreatedform').getForm().getValues();
  }
});
